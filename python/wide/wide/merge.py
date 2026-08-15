"""merge 核心：小宽表 → 大宽表。

纯函数（不读写文件），CLI/2a 脚本只负责调用与落盘。
优先级语义：行级裁决——new 行的 source 优先级 >= old 行时，new 的同名
字段覆盖；否则 new 只补缺。surfaces/mediums/equivs/sources 无论谁赢都并集。
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
import time
from typing import Literal, Optional

from .schema import Equiv, Row, Wide, SCHEMA

# 互斥位组：光泽度。merge 时组内多位亮起视为数据冲突（人为报告，仍按规则应用）
MUTEX_GROUPS: dict[str, list[int]] = {"surface_gloss": [1 << 0, 1 << 1, 1 << 2]}

# 增量更新允许缺失的字段（有 key 即可），但 wide cli 应统计并 warn
# brand/code/serie 缺失 = 空串；color/bases 缺失 = None；surfaces/mediums 缺失 = 0
REQUIRED_FIELDS: tuple[str, ...] = ("brand", "serie", "code", "color", "bases", "surfaces", "mediums")


@dataclass
class DanglingResult:
    total: int
    rows_missing: int
    counts: dict[str, int]  # 字段 -> 缺失行数
    samples: dict[str, list[str]]  # 字段 -> 最多 3 个示例


def check_dangling(wide: Wide) -> DanglingResult:
    counts: dict[str, int] = {f: 0 for f in REQUIRED_FIELDS}
    samples: dict[str, list[str]] = {f: [] for f in REQUIRED_FIELDS}
    rows_missing = 0
    for r in wide.paints:
        row_missing = []
        for f in REQUIRED_FIELDS:
            v = getattr(r, f)
            if f in ("surfaces", "mediums"):
                missing = v == 0
            elif isinstance(v, str):
                missing = not v
            else:
                missing = v is None
            if missing:
                counts[f] += 1
                row_missing.append(f)
                if len(samples[f]) < 3:
                    samples[f].append(f"{r.brand}:{r.code}")
        if row_missing:
            rows_missing += 1
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


def _channel_delta(a: int, b: int) -> int:
    return sum(abs(((a >> s) & 0xFF) - ((b >> s) & 0xFF)) for s in (16, 8, 0))


def row_priority(row: Row, priority: dict[str, int]) -> int:
    """行的优先级 = 其各来源 priority 的最大值（未登记 = 0）。"""
    return max([priority.get(s, 0) for s in row.sources] or [0])


def _mutex_conflict(flags: dict[str, int], group_name: str) -> list[str]:
    bits = MUTEX_GROUPS[group_name]
    lit = [b for b in bits if flags & b]
    return [group_name] if len(lit) > 1 else []


def merge_row(old: Optional[Row], new: Row, priority: dict[str, int],
              now: int | None = None) -> tuple[Row, RowDiff]:
    now = now or int(time.time())
    if old is None:
        return new.model_copy(update={"created_ts": now, "update_ts": now}), \
            RowDiff(new.brand, new.code, "new")

    new_wins = row_priority(new, priority) >= row_priority(old, priority)
    # 新行除 equivs 外无任何其他数据（增量补充等价声明）：其 sources 只由等价
    # 来源组成，不并入行的 sources（等价来源由 equiv.source 承载，见 schema 约定）
    if _only_equivs(new):
        merged_sources = old.sources
    else:
        merged_sources = _dedupe(old.sources + new.sources)
    # 同名语言：new_wins 时 new 覆盖，否则 old 覆盖（另一个补缺）
    src = {**old.desc, **new.desc} if new_wins else {**new.desc, **old.desc}
    color = (new.color if new.color is not None else old.color) if new_wins else (
        old.color if old.color is not None else new.color)
    bases = (new.bases if new.bases is not None else old.bases) if new_wins else (
        old.bases if old.bases is not None else new.bases)
    serie = (new.serie if new.serie is not None else old.serie) if new_wins else (
        old.serie if old.serie is not None else new.serie)

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
        created_ts=old.created_ts,
        update_ts=old.update_ts,
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

    # 冲突检测
    if old.serie and new.serie and old.serie != new.serie:
        conflicts.append(f"{old.brand}:{old.code} serie {old.serie} -> {new.serie}")
    if old.color and new.color and old.color != new.color:
        delta = _channel_delta(old.color, new.color)
        if delta > MergeFlags().color_delta:
            conflicts.append(
                f"{old.brand}:{old.code} color #{old.color:06X} -> #{new.color:06X} "
                f"(delta {delta} > {MergeFlags().color_delta})"
            )
    for g in MUTEX_GROUPS:
        conflicts += [f"{old.brand}:{old.code} mutex {c}" for c in _mutex_conflict(merged.surfaces, g)]
    if new_wins is False:
        fields.append(FieldDiff("priority", old.sources, new.sources, "rejected"))

    merged = merged.model_copy(update={"update_ts": now})
    return merged, RowDiff(old.brand, old.code, "updated", fields, conflicts)


def merge_wide(old: Optional[Wide], new: Wide, priority: dict[str, int],
               now: int | None = None) -> MergeResult:
    now = now or int(time.time())
    by_key = old.by_key() if old else {}
    diffs: list[RowDiff] = []
    conflicts: list[str] = []
    for row in new.paints:
        merged, diff = merge_row(by_key.get(row.key()), row, priority, now=now)
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
            and r.note is None)


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
