"""阶段3：大宽表 data/wide.json → app 数据文件。

产出（web/static/）：
- paints.bin      主数据，msgpack 列式（10 字段，Rust wasm 用前 7 列，JS 用全列）
- equivs.bin      等价表，msgpack 数组 [[src_idx, dst_idx], ...]（单向，仅源声明方向）
- paints/<lang>.json  i18n 名称字典（en/zh/ja/es 合并 wide 源语言 + 现有翻译；raw 重建）

设计约定：
- 行序 = (brand, serie, natural(code)) 排序，index 由此确定，wasm majors 与 equivs 共用
- surfaces 位转换：wide 是 FL=1<<5/PA=1<<6，wasm/前端是 PA=1<<5/FL=1<<6，生成时对调 bit5/6
- desc 只进 i18n（不进主数据）；equivs 独立文件；dangling equivs 跳过（不生成对）
- 不做传递闭包：equivs 只保留源直接声明的方向
"""

from __future__ import annotations

import json
import re
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

import msgpack

ROOT = Path(__file__).resolve().parents[1]
WIDE = ROOT / "data" / "wide.json"
OUT = ROOT / "web" / "static"
PAINTS_DIR = OUT / "paints"

SCHEMA = "paintbox.paints.v2"

# wide surfaces 位 -> wasm/前端位（PA/FL 对调）
_WIDE_TO_WASM = {1 << 0: 1 << 0, 1 << 1: 1 << 1, 1 << 2: 1 << 2, 1 << 3: 1 << 3,
                 1 << 4: 1 << 4, 1 << 5: 1 << 6, 1 << 6: 1 << 5, 1 << 7: 1 << 7,
                 1 << 8: 1 << 8}


def natural_key(s: str) -> list:
    return [int(t) if t.isdigit() else t for t in re.split(r"(\d+)", s)]


# 全大写英文名 sanitize 时的保留缩写（系列/色卡/军队标识，长度 >= 3 才需要列出）
_EN_KEEP = {"RLM", "IJA", "IJN", "IDF", "GGX", "SEED", "WW", "JASDF", "JMSDF", "RAF", "USMC", "ADC", "PRU", "AII", "AMT", "NATO", "LAF", "SLA", "CARC", "MERDC", "RAL"}


def sanitize_en(name: str) -> str:
    """全大写的官方英文名 -> Title Case（如 TIRE BLACK -> Tire Black）。

    判断：大写字母占比 >= 0.5 视为"全大写官方名"才处理（避免破坏已 Title
    Case / 混合格式，如 Wings of Light Holo Blue、RX-78 WHITE Ver. 除外）。
    token 规则：含数字（RLM02/FS36176）或长度 <= 2（MS/GX/UV）保留；
    _EN_KEEP 中的缩写保留；其余 capitalize（GRAY -> Gray）。
    """
    letters = [ch for ch in name if ch.isalpha()]
    if not letters or sum(ch.isupper() for ch in letters) / len(letters) < 0.5:
        return name

    def fix(tok: str) -> str:
        if not tok or any(ch.isdigit() for ch in tok):
            return tok
        if tok.isupper():
            if len(tok) <= 2 or tok in _EN_KEEP:
                return tok
            return tok.capitalize()
        return tok  # 已含小写（'s/Ver.）-> 保持原样

    return "".join(fix(t) for t in re.split(r"([^A-Za-z]+)", name))


def to_wasm_surfaces(v: int) -> int:
    out = 0
    for bit in (1 << i for i in range(9)):
        if v & bit:
            out |= _WIDE_TO_WASM[bit]
    return out


def load_wide() -> dict:
    return json.loads(WIDE.read_text(encoding="utf-8"))


def main() -> None:
    wide = load_wide()
    rows = wide["paints"]

    # 行序（稳定 + 自然序）
    rows.sort(key=lambda r: (r["brand"], r["serie"] or "", natural_key(r["code"])))
    index_of = {(r["brand"], r["code"]): i for i, r in enumerate(rows)}
    n = len(rows)

    # 字典
    brands = sorted({r["brand"] for r in rows})
    series = sorted({(r["brand"], r["serie"]) for r in rows if r["serie"]})
    series_list = [s for _, s in series]
    sources_all = sorted({s for r in rows for s in r.get("sources", [])})
    brand_id = {b: i for i, b in enumerate(brands)}
    serie_id = {s: i for i, s in enumerate(series_list)}
    source_id = {s: 1 << i for i, s in enumerate(sources_all)}

    cols = {k: [] for k in ("brand", "serie", "code", "color", "bases",
                            "surfaces", "mediums", "sources", "updated")}
    stats = Counter()
    for r in rows:
        cols["brand"].append(brand_id[r["brand"]])
        cols["serie"].append(serie_id.get(r.get("serie"), 0))
        cols["code"].append(r["code"])
        cols["color"].append(r.get("color") or 0)
        cols["bases"].append((r.get("bases") or 0) & 0xFFFF)
        cols["surfaces"].append(to_wasm_surfaces(r.get("surfaces") or 0) & 0xFFFF)
        cols["mediums"].append((r.get("mediums") or 0) & 0xFFFF)
        mask = 0
        for s in r.get("sources", []):
            mask |= source_id[s]
        cols["sources"].append(mask)
        cols["updated"].append(r.get("update_ts") or 0)
        stats[r["brand"]] += 1

    blob = [
        SCHEMA, n,
        brands, series_list, sources_all,
        cols["brand"], cols["serie"], cols["code"],
        cols["color"], cols["bases"], cols["surfaces"], cols["mediums"],
        cols["sources"], cols["updated"],
    ]
    (OUT / "paints.bin").write_bytes(msgpack.packb(blob, use_bin_type=False))
    print(f"paints.bin: {len(msgpack.packb(blob, use_bin_type=False))} bytes, {n} rows, "
          f"brands={dict(stats)}")

    # equivs.bin：单向对（仅源声明方向），每条带声明来源 source id
    # 格式：[n_pairs, dict_sources, src_idx[], dst_idx[], source_id[]]
    # 双向由前端 JS 解析时补充（反向继承 source）；不做传递闭包
    # dangling 跳过（目标不在当前行集）；旧 equivalences.csv 历史坏数据不转换
    pairs = []
    pair_sources = []
    skipped = 0
    equiv_sources = sorted({e["source"] for r in rows for e in r.get("equivs", [])})
    equiv_src_id = {s: i for i, s in enumerate(equiv_sources)}
    for i, r in enumerate(rows):
        for e in r.get("equivs", []):
            dst = index_of.get((e["brand"], e["code"]))
            if dst is None:
                skipped += 1
                continue
            pairs.append((i, dst))
            pair_sources.append(equiv_src_id[e["source"]])
    equiv_blob = [len(pairs), equiv_sources,
                  [a for a, _ in pairs], [b for _, b in pairs], pair_sources]
    (OUT / "equivs.bin").write_bytes(msgpack.packb(equiv_blob, use_bin_type=False))
    print(f"equivs.bin: {len(msgpack.packb(equiv_blob, use_bin_type=False))} bytes, "
          f"{len(pairs)} pairs, sources={equiv_sources} (skipped {skipped} dangling)")

    # i18n：合并 wide 源语言 + 现有翻译；raw 重建
    old = {}
    for lang in ("en", "zh", "ja", "es"):
        p = PAINTS_DIR / f"{lang}.json"
        old[lang] = json.loads(p.read_text(encoding="utf-8")) if p.exists() else {}
    raw: dict[str, dict] = {}
    missing_zh: list[tuple[str, str]] = []
    for r in rows:
        b, c = r["brand"], r["code"]
        for lang, name in (r.get("desc") or {}).items():
            raw.setdefault(lang, {}).setdefault(b, {})[c] = name  # raw 保持源语言原文
            if lang == "en":
                name = sanitize_en(name)  # 官方全大写 -> Title Case（仅展示文件）
            if lang in old:
                old[lang].setdefault(b, {})[c] = name
        if "zh" not in (r.get("desc") or {}) and c not in old["zh"].get(b, {}):
            missing_zh.append((b, c))
    for lang in ("en", "zh", "ja", "es"):
        (PAINTS_DIR / f"{lang}.json").write_text(
            json.dumps(old[lang], ensure_ascii=False, indent=1),
            encoding="utf-8")
    (PAINTS_DIR / "raw.json").write_text(
        json.dumps(raw, ensure_ascii=False, indent=1), encoding="utf-8")
    for lang in ("en", "zh", "ja", "es"):
        print(f"  paints/{lang}.json: {sum(len(v) for v in old[lang].values())} names")
    print(f"  paints/raw.json: {sum(len(v) for b in raw.values() for v in b.values())} names")
    print(f"[warn] zh 缺口（新色无中文名）: {len(missing_zh)}（如 {missing_zh[:3]}）")

    # 统计
    print(f"\n生成完成 {datetime.now(timezone.utc).isoformat(timespec='seconds')}")


if __name__ == "__main__":
    main()
