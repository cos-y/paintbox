"""merge 核心：小宽表 → 大宽表。

纯函数（不读写文件），CLI/2a 脚本只负责调用与落盘。
优先级语义：行级裁决——new 行的 source 优先级 >= old 行时，new 的同名
字段覆盖；否则 new 只补缺。surfaces/mediums/equivs/sources 无论谁赢都并集。
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import datetime, timezone
import time
from typing import Literal, Optional

from .schema import Equiv, Row, Wide, SCHEMA

# 互斥位组：光泽度。merge 时组内多位亮起视为数据冲突（人为报告，仍按规则应用）
MUTEX_GROUPS: dict[str, list[int]] = {"surface_gloss": [1 << 0, 1 << 1, 1 << 2]}

# 禁止冲突的 replace 键（替换式语义，冲突行为由 merge 的 conflict 参数指定：
# update=新值覆盖 / ignore=旧值保留 / error=报错退出）。当前只有 extra；
# 后续新增此类键时在此登记。
REPLACE_CONFLICT_KEYS: tuple[str, ...] = ("extra",)
# 允许的 conflict 取值
CONFLICT_MODES = ("update", "ignore", "error")

# 增量更新允许缺失的字段（有 key 即可），但 wide cli 应统计并 warn
# brand/code/serie 缺失 = 空串；color/bases 缺失 = None；surfaces/mediums 缺失 = 0
REQUIRED_FIELDS: tuple[str, ...] = ("brand", "serie", "code", "color", "bases", "surfaces", "mediums")


@dataclass
class DanglingResult:
    total: int
    rows_missing: int
    counts: dict[str, int]  # 字段 -> 缺失行数
    samples: dict[str, list[str]]  # 字段 -> 最多 3 个示例


def missing_fields(r: Row) -> list[str]:
    """该行缺失的必需字段（与 check_dangling 同一判定）。

    判定：brand/serie/code 空串、color/bases None、surfaces/mediums 0。
    """
    missing: list[str] = []
    for f in REQUIRED_FIELDS:
        v = getattr(r, f)
        if f in ("surfaces", "mediums"):
            ok = v != 0
        elif isinstance(v, str):
            ok = bool(v)
        else:
            ok = v is not None
        if not ok:
            missing.append(f)
    return missing


def partial_data(r: Row) -> bool:
    """color/bases/surfaces/mediums 部分提供（有值但不全）→ 数据不完整。

    纯补充源（gunze26 类：只带 code/equivs，四维全空）合法；
    半提供（cc070 类：有 color 却无 bases/surfaces/mediums）拒绝。
    """
    vals = [r.color is not None, r.bases is not None,
            r.surfaces != 0, r.mediums != 0]
    return any(vals) and not all(vals)


def check_dangling(wide: Wide) -> DanglingResult:
    counts: dict[str, int] = {f: 0 for f in REQUIRED_FIELDS}
    samples: dict[str, list[str]] = {f: [] for f in REQUIRED_FIELDS}
    rows_missing = 0
    for r in wide.paints:
        row_missing = missing_fields(r)
        if row_missing:
            rows_missing += 1
            for f in row_missing:
                counts[f] += 1
                if len(samples[f]) < 3:
                    samples[f].append(f"{r.brand}:{r.code}")
    return DanglingResult(total=len(wide.paints), rows_missing=rows_missing,
                          counts=counts, samples=samples)


@dataclass
class MergeFlags:
    color_delta: int = 12  # 颜色冲突阈值：RGB 通道差绝对值之和


@dataclass
class FieldDiff:
    field: str
    old: object
    new: object
    kind: Literal["added", "updated", "rejected", "conflict"]


@dataclass
class RowDiff:
    brand: str
    code: str
    kind: Literal["new", "updated", "unchanged"]
    fields: list[FieldDiff] = None
    conflicts: list[str] = None

    def __post_init__(self):
        self.fields = self.fields or []
        self.conflicts = self.conflicts or []


@dataclass
class MergeResult:
    wide: Wide
    report: "MergeReport"


@dataclass
class MergeReport:
    new_rows: int = 0
    updated_rows: int = 0
    unchanged_rows: int = 0
    conflicts: list[str] = None
    diffs: list[RowDiff] = None

    def __post_init__(self):
        self.conflicts = self.conflicts or []
        self.diffs = self.diffs or []


@dataclass
class ConflictSummary:
    """冲突分类统计（供 CLI 报告，纯函数 summarize_conflicts 产出）。"""

    by_type: dict[str, int]  # serie / color / mutex -> 计数
    color_delta: list[int]  # color 冲突的 delta（升序）
    desc_diffs: list[tuple[str, str, str]]  # (行key, lang, kind)
    samples: dict[str, list[str]]  # 每类最多 3 条原始 conflict

    def __post_init__(self):
        self.color_delta = sorted(self.color_delta)
        self.desc_diffs = list(self.desc_diffs)


def summarize_conflicts(report: MergeReport) -> ConflictSummary:
    """把 report 的 conflicts 字符串与 diffs 的 desc 差异分类统计。

    冲突字符串格式（merge_row 生成）：
      "brand:code serie X -> Y" / "brand:code color #A -> #B (delta N)" /
      "brand:code mutex GROUP"
    desc 措辞差异不在 conflicts 里，而在 diffs[].fields 的 desc.<lang>
    （overwrite=True 时 kind=updated 新覆盖，False 时 kind=rejected 保留）。
    """
    import re as _re

    by_type: dict[str, int] = {"serie": 0, "color": 0, "mutex": 0, "extra": 0}
    deltas: list[int] = []
    samples: dict[str, list[str]] = {"serie": [], "color": [], "mutex": []}
    for c in report.conflicts:
        parts = c.split()
        t = parts[1] if len(parts) > 1 else "?"
        if t not in by_type:
            by_type[t] = 0
        by_type[t] += 1
        if t == "color":
            m = _re.search(r"\(delta (\d+)", c)
            if m:
                deltas.append(int(m.group(1)))
        if len(samples.setdefault(t, [])) < 3:
            samples[t].append(c)

    desc_diffs: list[tuple[str, str, str]] = []
    for d in report.diffs:
        for f in d.fields or []:
            if f.field.startswith("desc.") and f.kind in ("updated", "rejected"):
                desc_diffs.append((f"{d.brand}:{d.code}", f.field[len("desc."):], f.kind))
    return ConflictSummary(by_type=by_type, color_delta=deltas,
                           desc_diffs=desc_diffs, samples=samples)


def _channel_delta(a: int, b: int) -> int:
    return sum(abs(((a >> s) & 0xFF) - ((b >> s) & 0xFF)) for s in (16, 8, 0))


def _adopted(old: Row, new: Row, color, bases, serie, desc) -> bool:
    """主体数据（color/bases/serie/desc）是否有 new 的值实际生效。

    new 与 old 全同（或 --keep 下冲突列全部保留）→ 不采纳 → sources 不并入；
    补缺（old 缺字段被 new 补上）也算采纳。仅 union 了 surfaces/mediums/
    equivs 不算（equivs 来源由 equiv.source 承载）。
    """
    for f, v in (("color", color), ("bases", bases), ("serie", serie)):
        n, o = getattr(new, f), getattr(old, f)
        if n is not None and n != o and v == n:
            return True
    for lang, v in desc.items():
        if v != old.desc.get(lang) and new.desc.get(lang) == v:
            return True
    return False


def _mutex_conflict(flags: dict[str, int], group_name: str) -> list[str]:
    bits = MUTEX_GROUPS[group_name]
    lit = [b for b in bits if flags & b]
    return [group_name] if len(lit) > 1 else []


def merge_row(old: Optional[Row], new: Row, overwrite: bool = True,
              conflict: str = "error") -> tuple[Row, RowDiff]:
    """合并一行。overwrite=True（默认）时新行覆盖旧行的同名冲突字段；
    overwrite=False 时旧列优先（只补缺，surfaces/mediums/equivs/sources 仍 union）。
    conflict 指定禁止冲突的 replace 键（REPLACE_CONFLICT_KEYS）在双方都有值且
    不同时的行为：update=新值覆盖 / ignore=旧值保留 / error=记入 conflicts（调用方报错退出）。
    更新时间不入文件（由 build_data 从 git 派生）。
    """
    if old is None:
        return new, RowDiff(new.brand, new.code, "new")

    def pick(newv, oldv):
        return (newv if newv is not None else oldv) if overwrite else (
            oldv if oldv is not None else newv)

    # 同名语言：overwrite 时 new 覆盖，否则 old 覆盖（另一个补缺）
    src = {**old.desc, **new.desc} if overwrite else {**new.desc, **old.desc}
    color = pick(new.color, old.color)
    bases = pick(new.bases, old.bases)
    serie = pick(new.serie, old.serie)

    # 新行除 equivs 外无任何其他数据（增量补充等价声明）：其 sources 只由等价
    # 来源组成，不并入行的 sources（等价来源由 equiv.source 承载，见 schema 约定）
    # 主体数据（color/bases/serie/desc）未被采纳时 sources 也不并入
    if _only_equivs(new) or not _adopted(old, new, color, bases, serie, src):
        merged_sources = old.sources
    else:
        merged_sources = _dedupe(old.sources + new.sources)

    # 禁止冲突的 replace 键：双方都有值且不同 -> 按 conflict 参数处理
    extra = old.extra if old.extra is not None else new.extra
    extra_conflict = (old.extra is not None and new.extra is not None
                      and new.extra != old.extra)
    if extra_conflict and conflict != "ignore":
        extra = new.extra

    merged = Row(
        brand=old.brand,
        serie=serie,
        code=old.code,
        color=color,
        desc=src,
        equivs=_dedupe_keys(old.equivs + new.equivs),
        bases=bases,
        surfaces=old.surfaces | new.surfaces,
        mediums=old.mediums | new.mediums,
        sources=merged_sources,
        note=old.note or new.note,
        extra=extra,
    )

    fields: list[FieldDiff] = []
    conflicts: list[str] = []
    if merged == old:
        return old, RowDiff(old.brand, old.code, "unchanged")

    # 逐字段变更明细（新行对旧行的增量）
    for f in ("color", "bases", "serie"):
        o, n = getattr(old, f), getattr(merged, f)
        if n != o:
            fields.append(FieldDiff(f, o, n, "updated"))
    for lang in sorted(set(new.desc) | set(old.desc)):
        o, n = old.desc.get(lang), merged.desc.get(lang)
        if n != o:
            fields.append(FieldDiff(f"desc.{lang}", o, n, "added" if o is None else "updated"))
    new_equivs = [e for e in merged.equivs if e not in old.equivs]
    if new_equivs:
        fields.append(FieldDiff("equivs", len(old.equivs), len(merged.equivs), "added"))
    for f in ("surfaces", "mediums"):
        o, n = getattr(old, f), getattr(merged, f)
        if n != o:
            fields.append(FieldDiff(f, o, n, "updated"))
    if merged.extra != old.extra:
        kind = "added" if old.extra is None else (
            "rejected" if extra_conflict and conflict == "ignore" else "updated")
        fields.append(FieldDiff("extra", old.extra, merged.extra, kind))

    # 冲突检测
    if old.serie and new.serie and old.serie != new.serie:
        conflicts.append(f"{old.brand}:{old.code} serie {old.serie} -> {new.serie}")
    if extra_conflict:
        fmt = lambda v: json.dumps(v, ensure_ascii=False, separators=(",", ":"))
        conflicts.append(f"{old.brand}:{old.code} extra {fmt(old.extra)} -> {fmt(new.extra)}")
    if old.color and new.color and old.color != new.color:
        delta = _channel_delta(old.color, new.color)
        if delta > MergeFlags().color_delta:
            conflicts.append(
                f"{old.brand}:{old.code} color #{old.color:06X} -> #{new.color:06X} "
                f"(delta {delta} > {MergeFlags().color_delta})"
            )
    for g in MUTEX_GROUPS:
        conflicts += [f"{old.brand}:{old.code} mutex {c}" for c in _mutex_conflict(merged.surfaces, g)]
    # overwrite=False 时被拒绝的覆盖（旧列保留，只报告）
    if not overwrite:
        for f in ("color", "bases", "serie"):
            o, n = getattr(old, f), getattr(new, f)
            if n is not None and n != o:
                fields.append(FieldDiff(f, o, n, "rejected"))
        for lang in sorted(set(new.desc) & set(old.desc)):
            if new.desc[lang] != old.desc[lang]:
                fields.append(FieldDiff(f"desc.{lang}", old.desc[lang], new.desc[lang], "rejected"))

    return merged, RowDiff(old.brand, old.code, "updated", fields, conflicts)


def merge_wide(old: Optional[Wide], new: Wide, overwrite: bool = True,
               conflict: str = "error") -> MergeResult:
    by_key = old.by_key() if old else {}
    diffs: list[RowDiff] = []
    conflicts: list[str] = []
    for row in new.paints:
        merged, diff = merge_row(by_key.get(row.key()), row, overwrite, conflict)
        by_key[row.key()] = merged
        diffs.append(diff)
        conflicts += diff.conflicts

    # 行序 = (brand, serie, code) 升序（稳定的排序约定，与数据源顺序无关）
    paints = sorted(by_key.values(), key=lambda r: (r.brand, r.serie or "", r.code))
    report = MergeReport(
        new_rows=sum(1 for d in diffs if d.kind == "new"),
        updated_rows=sum(1 for d in diffs if d.kind == "updated"),
        unchanged_rows=sum(1 for d in diffs if d.kind == "unchanged"),
        conflicts=conflicts,
        diffs=diffs,
    )
    return MergeResult(
        Wide(schema_name=SCHEMA,
             generatedAt=datetime.now(timezone.utc).isoformat(),
             paints=paints),
        report,
    )


def _only_equivs(r: Row) -> bool:
    """该行是否只携带等价声明（无其他数据字段）。"""
    return (r.serie is None and r.color is None and not r.desc
            and r.bases is None and r.surfaces == 0 and r.mediums == 0
            and r.note is None and r.extra is None)


def _dedupe(items: list[str]) -> list[str]:
    return list(dict.fromkeys(items))


def _dedupe_keys(items: list[Equiv]) -> list[Equiv]:
    out: list[Equiv] = []
    seen = set()
    for k in items:
        key = (k.brand, k.code, k.source)
        if key not in seen:
            seen.add(key)
            out.append(k)
    return out
