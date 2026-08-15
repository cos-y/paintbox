"""wide CLI：小宽表 merge 进大宽表。

用法：
  wide merge raw/gunze/01ksp_2026/wide.json [--source raw/gunze/01ksp_2026/source.json] \
        [--priority ksp2026=10,...] [--apply]

第一个参数是小宽表路径（相对项目根）。
--source 指定该源的 meta（默认小宽表同目录 source.json），注册进大宽表顶层 sources。
默认 dry-run（打印报告不写文件）；--apply 才写大宽表（默认 raw/wide.json）。
优先级：--priority 指定（source_id=数字），未指定的 source 优先级为 0。
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

from .merge import DanglingResult, MergeReport, check_dangling, merge_wide
from .schema import Wide

def _project_root() -> Path:
    """向上找含 web/ 目录的项目根（不依赖包嵌套层级）。"""
    return next(p for p in Path(__file__).resolve().parents if (p / "web").is_dir())

ROOT = _project_root()
DEFAULT_WIDE = ROOT / "data" / "wide.json"


def _load_wide(path: Path) -> Wide | None:
    if not path.exists():
        return None
    return Wide.model_validate_json(path.read_text(encoding="utf-8"))


def _format_wide(wide: Wide) -> str:
    """头部（schema/generatedAt/sources）pretty，paints 每条记录一行。

    git diff 时一行 = 一条记录（可审计），行数 ~= 记录数而非记录×20。
    """
    data = json.loads(wide.model_dump_json(by_alias=True))
    head = {k: v for k, v in data.items() if k != "paints"}
    head_str = json.dumps(head, ensure_ascii=False, indent=1)
    paints_str = ",\n".join(
        json.dumps(p, ensure_ascii=False, separators=(",", ":"))
        for p in data["paints"])
    return "{\n" + head_str[1:-1] + ",\n  \"paints\": [\n" + paints_str + "\n  ]\n}"


def _save_wide(wide: Wide, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(_format_wide(wide), encoding="utf-8")
    # 自校验：写出的文件必须能被 schema 读回
    Wide.model_validate_json(path.read_text(encoding="utf-8"))


def check_equivs(wide) -> list[tuple[tuple[str, str], str, str]]:
    """审计：所有 equivs 中目标记录不在宽表中的（含历史数据，不限于本次 merge）。"""
    keys = {r.key() for r in wide.paints}
    bad = []
    for r in wide.paints:
        for e in r.equivs:
            if (e.brand, e.code) not in keys:
                bad.append((r.key(), e.brand, e.code))
    return bad


def print_equiv_audit(bad, brand: str | None = None) -> None:
    from collections import Counter, defaultdict
    if not bad:
        print("[ok] all equivs reference existing records")
        return
    if brand:
        items = [(rk, rc, tc) for (rk, rc), tb, tc in bad if tb == brand]
        if not items:
            print(f"[ok] no dangling equivs targeting {brand}")
            return
        by = defaultdict(list)
        for rk, rc, tc in items:
            by[tc].append(f"{rk}:{rc}")
        print(f"[dangling equivs] {len(items)} equivs targeting {brand}:")
        for tc in sorted(by):
            refs = by[tc]
            shown = ", ".join(refs[:6])
            if len(refs) > 6:
                shown += f" ... (+{len(refs) - 6})"
            print(f"  {brand}:{tc}  <- {shown}")
        return
    by_target = Counter((b, c) for _, b, c in bad)
    by_brand = Counter(b for _, b, _ in bad)
    print(f"[dangling equivs] {len(bad)} equivs reference non-existent records")
    print(f"  by target brand: {dict(by_brand.most_common())}")
    print("  top targets:")
    for (b, c), n in by_target.most_common(10):
        print(f"    {b}:{c}  x{n}")
    print("  source rows (first 10):")
    for (rk, rc), tb, tc in bad[:10]:
        print(f"    {rk}:{rc} -> {tb}:{tc}")
    print(f"  (use --brand <name> to list all dangling equivs of one brand)")


def _rename_source(wide: Wide, old_id: str, new_id: str) -> Wide:
    """把宽表中引用 old_id 的所有行/等价声明重命名为 new_id（source 替换的一致性处理）。"""
    paints = []
    for r in wide.paints:
        update: dict = {}
        if old_id in r.sources:
            update["sources"] = list(
                dict.fromkeys(new_id if s == old_id else s for s in r.sources))
        if any(e.source == old_id for e in r.equivs):
            update["equivs"] = [
                e.model_copy(update={"source": new_id}) if e.source == old_id else e
                for e in r.equivs]
        if update:
            update["update_ts"] = int(time.time())
            r = r.model_copy(update=update)
        paints.append(r)
    return wide.model_copy(update={"paints": paints})


def _fmt(v) -> str:
    if isinstance(v, int):
        return f"#{v:06X}" if v > 0xFFFFFF else str(v)
    if isinstance(v, list):
        return f"[{len(v)} items]"
    return str(v)


def print_report(report: MergeReport, *, show_diffs: bool = True) -> None:
    print(f"new: {report.new_rows}   updated: {report.updated_rows}   "
          f"unchanged: {report.unchanged_rows}")
    if report.conflicts:
        print(f"\n[conflicts] {len(report.conflicts)}")
        for c in report.conflicts[:30]:
            print(f"  ! {c}")
        if len(report.conflicts) > 30:
            print(f"  ... and {len(report.conflicts) - 30} more")
    if show_diffs:
        for d in report.diffs:
            if d.kind == "unchanged":
                continue
            head = f"[{d.kind}] {d.brand}:{d.code}"
            if not d.fields and not d.conflicts:
                print(f"  {head}")
                continue
            print(f"  {head}")
            for f in d.fields:
                mark = "~" if f.kind == "rejected" else ("+" if f.kind == "added" else " ")
                print(f"    {mark} {f.field}: {_fmt(f.old)} -> {_fmt(f.new)}")


def print_dangling(d: DanglingResult) -> None:
    if d.rows_missing == 0:
        print(f"[ok] required fields complete: {d.total} rows")
        return
    print(f"[dangling] {d.rows_missing}/{d.total} rows missing required fields")
    for f in ("brand", "serie", "code", "color", "bases", "surfaces", "mediums"):
        c = d.counts[f]
        if c:
            pct = f"{100 * c / d.total:.0f}%" if d.total else "0%"
            print(f"  - {f}: {c} ({pct})  e.g. {', '.join(d.samples[f])}")


def _dangling_equivs(wide, old) -> list[tuple[tuple[str, str], str, str]]:
    """本次 merge 新增的 equivs 中，目标记录在结果宽表中不存在的。

    返回 [(行 key, 目标 brand, 目标 code)]；只统计 merge 添加的 equivs
    （新增行全部算，更新行只算 old 没有的），不扫描 wide 已有记录。
    """
    keys = {r.key() for r in wide.paints}
    old_rows = old.by_key() if old else {}
    dangling = []
    for r in wide.paints:
        old_row = old_rows.get(r.key())
        old_eq = {(e.brand, e.code) for e in (old_row.equivs if old_row else [])}
        for e in r.equivs:
            if (e.brand, e.code) in old_eq:
                continue  # 已有记录，不校验
            if (e.brand, e.code) not in keys:
                dangling.append((r.key(), e.brand, e.code))
    return dangling


def _resolve_dangling(dangling) -> str:
    """交互式三选一：keep / drop / abort。"""
    from collections import Counter
    by_brand = Counter(b for _, b, _ in dangling)
    print(f"\n[dangling equivs] {len(dangling)} new equivs reference non-existent records:")
    for brand, n in by_brand.most_common():
        print(f"  {brand}: {n}")
    for (bk, bc), tb, tc in dangling[:10]:
        print(f"    {bk[0]}:{bk[1]} -> {tb}:{tc}")
    if len(dangling) > 10:
        print(f"    ... and {len(dangling) - 10} more")
    while True:
        ans = input("resolve: [1] keep as dangling  [2] drop them  [3] abort: ").strip()
        if ans in ("1", "2", "3"):
            return ans
        print("  please enter 1, 2 or 3")


def main() -> None:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    ap = argparse.ArgumentParser(prog="wide", description="wide table merge tool")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p_merge = sub.add_parser("merge", help="merge a source wide table into the wide table")
    p_merge.add_argument("new", help="source wide JSON (relative to project root)")
    p_merge.add_argument("--source", default="", help="source meta JSON (default: beside new wide)")
    p_merge.add_argument("--source-mode", choices=["append", "replace"], default=None,
                         help="source id conflict resolution (required when id changes)")
    p_merge.add_argument("--priority", default="", help="comma list src=num, e.g. ksp2026=10")
    p_merge.add_argument("--apply", action="store_true", help="write back the wide table")
    p_merge.add_argument("--wide", default=str(DEFAULT_WIDE), help="target wide JSON path")

    p_check = sub.add_parser("check", help="report dangling rows and equivs references")
    p_check.add_argument("wide", nargs="?", default=str(DEFAULT_WIDE), help="wide JSON path")
    p_check.add_argument("--brand", default="",
                         help="list all dangling equivs targeting this brand")

    p_format = sub.add_parser("format", help="reformat wide JSON (header pretty, one paint per line)")
    p_format.add_argument("wide", nargs="?", default=str(DEFAULT_WIDE), help="wide JSON path")

    args = ap.parse_args()
    if args.cmd == "check":
        wide = _load_wide(Path(args.wide))
        if wide is None:
            print(f"wide not found: {args.wide}")
            raise SystemExit(1)
        print_dangling(check_dangling(wide))
        print_equiv_audit(check_equivs(wide), args.brand or None)
        return

    if args.cmd == "format":
        path = Path(args.wide)
        wide = _load_wide(path)
        if wide is None:
            print(f"wide not found: {args.wide}")
            raise SystemExit(1)
        _save_wide(wide, path)
        print(f"formatted: {path}")
        return

    # merge
    new_path = Path(args.new)
    if not new_path.is_absolute():
        new_path = ROOT / new_path

    priority: dict[str, int] = {}
    for kv in filter(None, args.priority.split(",")):
        k, _, v = kv.partition("=")
        priority[k.strip()] = int(v)

    old = _load_wide(Path(args.wide))
    new = _load_wide(new_path)
    if new is None:
        print(f"wide not found: {new_path}")
        raise SystemExit(1)

    # 源 meta 注册：小宽表自带 sources + cli --source（默认同目录 source.json）覆盖/补充
    source_meta: dict = dict(new.sources) if new else {}
    sid: str | None = None
    src_path = Path(args.source) if args.source else new_path.parent / "source.json"
    if src_path.exists():
        meta = json.loads(src_path.read_text(encoding="utf-8"))
        sid = meta.get("id")
        if sid:
            source_meta[sid] = meta
        else:
            print(f"[warn] source.json has no 'id': {src_path}")

    old_sources: dict = old.sources if old else {}

    # source id 一致性：id 变化但指向同一源（同 url/title）时，要求用户显式处理
    renamed: tuple[str, str] | None = None
    if sid and sid not in old_sources:
        same = [k for k, v in old_sources.items()
                if v.get("url") and v.get("url") == meta.get("url")
                or (not meta.get("url") and v.get("title") and v.get("title") == meta.get("title"))]
        if same:
            old_id = same[0]
            if args.source_mode is None:
                print(f"[error] source id changed: {old_id} -> {sid} (same source)")
                print("  choose --source-mode append (keep both) or --source-mode replace "
                      "(rename registrations and row references)")
                raise SystemExit(1)
            if args.source_mode == "replace":
                renamed = (old_id, sid)

    result = merge_wide(old, new, priority)
    if renamed:
        # 删除被替换的旧注册，行引用旧 id -> 新 id
        old_sources = {k: v for k, v in old_sources.items() if k != renamed[0]}
        result.wide = _rename_source(result.wide, *renamed)
    result.wide = result.wide.model_copy(
        update={"sources": {**old_sources, **source_meta}})
    if renamed:
        print(f"[ok] source renamed: {renamed[0]} -> {renamed[1]} (rows updated)")

    # 引用校验：行 sources 应能在注册表（大宽表 + 本次）中找到
    known = set(result.wide.sources)
    missing = sorted({s for row in new.paints for s in row.sources} - known)
    if missing:
        print(f"[warn] rows reference unknown sources: {missing}")

    print(f"old rows: {len(old.paints) if old else 0}  ->  new rows: {len(result.wide.paints)}")
    print_report(result.report)
    print_dangling(check_dangling(result.wide))

    # 新增 equivs 的目标存在性校验（只对本次 merge 添加的 equivs 生效）
    dangling = _dangling_equivs(result.wide, old)
    if dangling:
        if args.apply:
            choice = _resolve_dangling(dangling)
            if choice == "3":
                print("aborted, wide.json unchanged")
                raise SystemExit(1)
            if choice == "2":
                drop = {(b, c) for _, b, c in dangling}
                paints = []
                for r in result.wide.paints:
                    if any((e.brand, e.code) in drop for e in r.equivs):
                        r = r.model_copy(update={
                            "equivs": [e for e in r.equivs if (e.brand, e.code) not in drop],
                            "update_ts": int(time.time())})
                    paints.append(r)
                result.wide = result.wide.model_copy(update={"paints": paints})
                print(f"[ok] dropped {len(drop)} dangling equivs")
        else:
            print(f"[dangling equivs] {len(dangling)} new equivs reference non-existent "
                  f"records (will prompt on --apply)")

    if args.apply:
        _save_wide(result.wide, Path(args.wide))
        print(f"applied: {args.wide}")
    else:
        print("\n(dry-run, no file written; add --apply to write)")


if __name__ == "__main__":
    main()
