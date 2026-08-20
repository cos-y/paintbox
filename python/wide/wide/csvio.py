"""wide 宽表 CSV 序列化（两个宽表都用 CSV 存储 paints）。

约定（用户指定）：
- 固定列 + desc 动态语言列（所有记录 desc key 的并集，排序后成列）+ equivs 列
- extra 列存 compact JSON（含逗号/引号时 csv 模块自动加引号转义，读取无损还原）；空 = 无
- 列表分隔符 |（元素之间）；字段分隔符 ;（元素内 brand;code;source）
- color 用 #RRGGBB（可读、可列编辑），导入转回 int
- 空字符串 = None / 空集合；desc 空语言列不产生 key
- meta（schema/generatedAt/sources 注册表）仍存 JSON：
  大宽表 data/wide.json（meta）+ data/wide.csv（paints）
  小宽表 source.json（meta）+ wide.csv（paints）
"""

from __future__ import annotations

import csv
import json
from pathlib import Path

from .schema import Equiv, Row, SCHEMA, Wide

# 固定列（desc 语言列动态生成，插在 extra 与 equivs 之间）
FIXED = [
    "brand", "serie", "code", "color", "bases", "surfaces", "mediums",
    "sources", "note", "extra", "equivs",
]
LIST_SEP = "|"  # 列表分隔（元素之间）
FIELD_SEP = ";"  # 字段分隔（元素内）


def _desc_cols(paints: list[Row]) -> list[str]:
    langs = sorted({lang for r in paints for lang in r.desc})
    return [f"desc_{l}" for l in langs]


def row_to_fields(r: Row, desc_cols: list[str]) -> dict[str, str]:
    f = {c: "" for c in FIXED}
    f["brand"] = r.brand
    f["serie"] = r.serie or ""
    f["code"] = r.code
    f["color"] = f"#{r.color:06X}" if r.color is not None else ""
    f["bases"] = str(r.bases) if r.bases is not None else ""
    f["surfaces"] = str(r.surfaces)
    f["mediums"] = str(r.mediums)
    f["sources"] = LIST_SEP.join(r.sources)
    f["note"] = r.note or ""
    # extra 存 compact JSON；含逗号/引号时由 csv 模块自动加引号转义，读取无损还原
    f["extra"] = json.dumps(r.extra, ensure_ascii=False, separators=(",", ":")) if r.extra else ""
    for dc in desc_cols:
        f[dc] = r.desc.get(dc[len("desc_"):], "")
    f["equivs"] = LIST_SEP.join(
        f"{e.brand}{FIELD_SEP}{e.code}{FIELD_SEP}{e.source}" for e in r.equivs)
    return f


def fields_to_row(f: dict[str, str]) -> Row:
    desc = {c[len("desc_"):]: f[c] for c in f
            if c.startswith("desc_") and f.get(c)}
    equivs = []
    for item in filter(None, (f.get("equivs") or "").split(LIST_SEP)):
        parts = item.split(FIELD_SEP)
        if len(parts) == 3:
            equivs.append(Equiv(brand=parts[0], code=parts[1], source=parts[2]))
    color = int(f["color"][1:], 16) if f.get("color") else None
    extra = None
    if f.get("extra"):
        try:
            parsed = json.loads(f["extra"])
        except json.JSONDecodeError as e:
            raise ValueError(
                f"extra JSON 解析失败 ({f.get('brand')}:{f.get('code')}): {e}") from e
        if not isinstance(parsed, dict):
            raise ValueError(
                f"extra 必须是 JSON 对象 ({f.get('brand')}:{f.get('code')}): {parsed!r}")
        extra = parsed or None  # 空对象 {} 视同无
    return Row(
        brand=f["brand"],
        serie=f.get("serie") or None,
        code=f["code"],
        color=color,
        desc=desc,
        equivs=equivs,
        bases=int(f["bases"]) if f.get("bases") else None,
        surfaces=int(f.get("surfaces") or 0),
        mediums=int(f.get("mediums") or 0),
        sources=[s for s in (f.get("sources") or "").split(LIST_SEP) if s],
        note=f.get("note") or None,
        extra=extra,
    )


def read_paints_csv(path: Path) -> list[Row]:
    with open(path, newline="", encoding="utf-8") as fh:
        return [fields_to_row(r) for r in csv.DictReader(fh)]


def write_paints_csv(path: Path, paints: list[Row]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    desc_cols = _desc_cols(paints)
    cols = FIXED[:-1] + desc_cols + [FIXED[-1]]
    with open(path, "w", newline="", encoding="utf-8") as fh:
        w = csv.DictWriter(fh, fieldnames=cols)
        w.writeheader()
        for r in paints:
            w.writerow(row_to_fields(r, desc_cols))


def save_wide(wide: Wide, meta_path: Path) -> None:
    """写 meta JSON（schema/generatedAt/sources）+ 同目录 wide.csv。"""
    meta_path.parent.mkdir(parents=True, exist_ok=True)
    meta = {k: v for k, v in json.loads(
        wide.model_dump_json(by_alias=True)).items() if k != "paints"}
    meta_path.write_text(json.dumps(meta, ensure_ascii=False, indent=1),
                         encoding="utf-8")
    write_paints_csv(meta_path.with_suffix(".csv"), wide.paints)


def load_wide(meta_path: Path) -> Wide | None:
    """读 meta JSON + 同目录 wide.csv；任一缺失返回 None。"""
    csv_path = meta_path.with_suffix(".csv")
    if not meta_path.exists() or not csv_path.exists():
        return None
    meta = json.loads(meta_path.read_text(encoding="utf-8"))
    paints = read_paints_csv(csv_path)
    return Wide(
        schema_name=meta.get("schema", SCHEMA),
        generatedAt=meta.get("generatedAt", ""),
        sources=meta.get("sources", {}),
        paints=paints,
    )
