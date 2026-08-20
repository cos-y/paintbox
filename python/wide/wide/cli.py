"""wide CLI：小宽表 merge 进大宽表。

用法：
  wide merge raw/gunze/gunze26/wide.csv [--source raw/gunze/gunze26/source.json] \
        [--keep] [--apply]
  wide extra --key brand:code --json extra.json   # 单行：json 为 extra 对象，已有值会 prompt
  wide extra --json batch.json                    # 批量：json = {brand:code: extra, ...}
        [--conflict prompt|replace|reject] [--merge-policy prompt|replace|reject]

第一个参数是小宽表 CSV 路径（相对项目根），也接受目录（自动找 wide.csv）。
--source 指定该源的 meta（默认小宽表同目录 source.json），注册进大宽表 sources。
默认 dry-run（打印报告不写文件）；--apply 才写大宽表（默认 data/wide.json meta + data/wide.csv）。
冲突语义：默认新数据覆盖同名冲突列（color/bases/serie/desc 同语言）；
  --keep 改为保留旧列（只补缺，surfaces/mediums/equivs/sources 始终 union）。
格式：两个宽表都是 CSV（data/wide.csv / raw/<brand>/<source>/wide.csv），
  meta 分开（大宽表 data/wide.json 存 schema/generatedAt/sources 注册表；
  小宽表用同目录 source.json）。分隔符：列表 |、字段 ;。
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

from .csvio import load_wide, read_paints_csv, save_wide
from .merge import (
    CONFLICT_MODES,
    DanglingResult,
    REQUIRED_FIELDS,
    MergeReport,
    check_dangling,
    merge_wide,
    missing_fields,
    partial_data,
    summarize_conflicts,
)
from .schema import Wide

def _project_root() -> Path:
    """向上找含 web/ 目录的项目根（不依赖包嵌套层级）。"""
    return next(p for p in Path(__file__).resolve().parents if (p / "web").is_dir())

ROOT = _project_root()
DEFAULT_WIDE = ROOT / "data" / "wide.json"


def _load_wide(path: Path) -> Wide | None:
    return load_wide(path)


def _save_wide(wide: Wide, path: Path) -> None:
    save_wide(wide, path)
    # 自校验：写出的 meta + CSV 必须能读回
    load_wide(path)


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
        from collections import Counter
        import statistics

        s = summarize_conflicts(report)
        print(f"\n[conflicts] {len(report.conflicts)}  "
              f"(serie {s.by_type.get('serie', 0)} / color {s.by_type.get('color', 0)}"
              f" / mutex {s.by_type.get('mutex', 0)} / extra {s.by_type.get('extra', 0)})")
        if s.color_delta:
            ds = s.color_delta
            print(f"  color delta: min {ds[0]} / median "
                  f"{statistics.median(ds):.0f} / max {ds[-1]} "
                  f"({len(ds)} 处)")
        if s.desc_diffs:
            langs = Counter(lang for _, lang, _ in s.desc_diffs)
            kinds = Counter(kind for _, _, kind in s.desc_diffs)
            print(f"  desc 措辞差异: {len(s.desc_diffs)} 处 "
                  f"(langs {dict(langs)}, {dict(kinds)})")
            for key, lang, kind in s.desc_diffs[:3]:
                mark = "覆盖" if kind == "updated" else "保留(旧)"
                print(f"    ~ {key} desc.{lang} {mark}")
        # 推荐动作：color 冲突多且 delta 大（疑似版本差异）→ 建议 --keep
        if s.by_type.get("color", 0) > 5 and (not s.color_delta or s.color_delta[-1] > 30):
            print("  建议: color 冲突多且 delta 大(疑似版本差异)，"
                  "可用 --keep 保留现有值，新源只补缺")
        for t in ("serie", "color", "mutex", "extra"):
            for c in s.samples.get(t, [])[:3]:
                print(f"  ! {c}")
        if len(report.conflicts) > 9:
            print(f"  ... and {len(report.conflicts) - 9} more (see diffs below)")
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
        print(f"    {bk}:{bc} -> {tb}:{tc}")
    if len(dangling) > 10:
        print(f"    ... and {len(dangling) - 10} more")
    while True:
        ans = input("resolve: [1] keep as dangling  [2] drop them  [3] abort: ").strip()
        if ans in ("1", "2", "3"):
            return ans
        print("  please enter 1, 2 or 3")


def _extra_command(args) -> None:
    """wide extra：更新一行或批量行的 extra 字段（见 main 的用法注释）。

    单行 --key：json 文件为 extra 对象本身；批量：json 为 {brand:code: extra} 字典。
    --conflict 控制已有 extra 时的行为（prompt 逐行问 y/n/m，m 触发 merge）；
    --merge-policy 控制 merge 键冲突（同 key 不同值）时的行为。
    """
    wide = _load_wide(Path(args.wide))
    if wide is None:
        print(f"wide not found: {args.wide}")
        raise SystemExit(1)

    jpath = Path(args.json)
    if not jpath.exists():
        print(f"[error] json not found: {jpath}")
        raise SystemExit(1)
    try:
        data = json.loads(jpath.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        print(f"[error] invalid JSON in {jpath}: {e}")
        raise SystemExit(1)

    # 解析目标：[(key, extra), ...]
    targets: list[tuple[str, dict]] = []
    if args.key:
        if not isinstance(data, dict):
            print(f"[error] {jpath} must contain a JSON object (the extra for {args.key})")
            raise SystemExit(1)
        targets.append((args.key, data))
    else:
        if not isinstance(data, dict):
            print(f"[error] batch mode needs a JSON object mapping brand:code -> extra: {jpath}")
            raise SystemExit(1)
        for k, v in data.items():
            if ":" not in k:
                print(f"[error] invalid batch key: {k} (expected brand:code)")
                raise SystemExit(1)
            if not isinstance(v, dict):
                print(f"[error] batch extra for {k} must be a JSON object, "
                      f"got {type(v).__name__}")
                raise SystemExit(1)
            targets.append((k, v))
    if not targets:
        print("[error] nothing to do (empty batch dict)")
        raise SystemExit(1)

    def _ask(prompt: str, *choices: str) -> str:
        while True:
            try:
                ans = input(prompt).strip().lower()
            except EOFError:
                print("\n[abort] input exhausted, wide untouched")
                raise SystemExit(1)
            if ans in choices:
                return ans
            print(f"  please enter {'/'.join(choices)}")

    fmt = lambda v: json.dumps(v, ensure_ascii=False, separators=(",", ":"))
    paints = list(wide.paints)
    rows = {(r.brand, r.code): i for i, r in enumerate(paints)}
    stats = {"set": 0, "merged": 0, "skipped": 0, "unchanged": 0, "not_found": 0}

    for key, new_extra in targets:
        brand, _, code = key.partition(":")
        i = rows.get((brand, code))
        if i is None:
            if args.key:
                print(f"[error] no row {key} in wide")
                raise SystemExit(1)
            print(f"[warn] no row {key} in wide, skipped")
            stats["not_found"] += 1
            continue
        row = paints[i]

        if new_extra == row.extra:
            print(f"  - {key} extra unchanged")
            stats["unchanged"] += 1
            continue
        if not row.extra:
            final, action = new_extra, "set"
        elif args.conflict == "replace":
            final, action = new_extra, "set"
        elif args.conflict == "reject":
            print(f"  - {key} extra conflict, skipped (--conflict reject): {fmt(row.extra)}")
            stats["skipped"] += 1
            continue
        else:  # prompt
            print(f"[extra] {key} already has {fmt(row.extra)}")
            print(f"  new: {fmt(new_extra)}")
            choice = _ask("  [y] overwrite  [n] keep old  [m] merge: ", "y", "n", "m")
            if choice == "n":
                stats["skipped"] += 1
                continue
            if choice == "y":
                final, action = new_extra, "set"
            else:
                final = dict(row.extra)
                for k, v in new_extra.items():
                    if k in final and final[k] != v:
                        if args.merge_policy == "replace":
                            final[k] = v
                        elif args.merge_policy == "reject":
                            pass
                        else:
                            c = _ask(f"    merge conflict on '{k}': [r] replace (new) "
                                     f"/ [j] reject (keep old): ", "r", "j")
                            if c == "r":
                                final[k] = v
                    else:
                        final[k] = v
                action = "merged" if final != row.extra else "unchanged"

        if final == row.extra:
            print(f"  - {key} extra unchanged")
            stats["unchanged"] += 1
            continue
        paints[i] = row.model_copy(update={"extra": final})
        stats[action] += 1
        print(f"  + {key} extra {action}: {fmt(final)}")

    if not (stats["set"] or stats["merged"]):
        print("[ok] nothing changed, wide untouched")
        return
    _save_wide(wide.model_copy(update={"paints": paints}), Path(args.wide))
    print(f"[ok] {stats['set']} set, {stats['merged']} merged, {stats['skipped']} skipped, "
          f"{stats['unchanged']} unchanged, {stats['not_found']} not found "
          f"(saved {args.wide})")


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
    p_merge.add_argument("--keep", action="store_true",
                         help="on conflicts keep existing columns (default: new data overwrites)")
    p_merge.add_argument("--conflict", choices=CONFLICT_MODES, default="error",
                         help="behavior for replace keys with both values set and different "
                              "(extra): update=new wins / ignore=old wins / error=abort "
                              "(default: error)")
    p_merge.add_argument("--apply", action="store_true", help="write back the wide table")
    p_merge.add_argument("--wide", default=str(DEFAULT_WIDE), help="target wide JSON path")

    p_check = sub.add_parser("check", help="report dangling rows and equivs references")
    p_check.add_argument("wide", nargs="?", default=str(DEFAULT_WIDE), help="wide JSON path")
    p_check.add_argument("--brand", default="",
                         help="list all dangling equivs targeting this brand")

    p_extra = sub.add_parser("extra",
                             help="set/merge the extra field of paints")
    p_extra.add_argument("--key", default="",
                         help="row to update, format brand:code (omit for batch mode)")
    p_extra.add_argument("--json", required=True,
                         help="JSON file: extra object (with --key), or "
                              "{brand:code: extra} dict (batch mode)")
    p_extra.add_argument("--conflict", choices=["prompt", "replace", "reject"],
                         default="prompt",
                         help="behavior when the row already has extra: prompt (ask "
                              "y/n/m per row) / replace (overwrite) / reject (skip)")
    p_extra.add_argument("--merge-policy", choices=["prompt", "replace", "reject"],
                         default="prompt",
                         help="behavior on merge key conflicts (same key, different value): "
                              "prompt (ask replace/reject) / replace (new wins) / "
                              "reject (keep old)")
    p_extra.add_argument("--wide", default=str(DEFAULT_WIDE), help="target wide JSON path")

    args = ap.parse_args()
    if args.cmd == "check":
        wide = _load_wide(Path(args.wide))
        if wide is None:
            print(f"wide not found: {args.wide}")
            raise SystemExit(1)
        print_dangling(check_dangling(wide))
        print_equiv_audit(check_equivs(wide), args.brand or None)
        return

    if args.cmd == "extra":
        _extra_command(args)
        return


    # merge
    new_path = Path(args.new)
    if not new_path.is_absolute():
        new_path = ROOT / new_path
    if new_path.is_dir():
        new_path = new_path / "wide.csv"

    old = _load_wide(Path(args.wide))
    if not new_path.exists():
        print(f"wide.csv not found: {new_path}")
        raise SystemExit(1)
    new_paints = read_paints_csv(new_path)

    # 输入校验：数据半提供（color/bases/surfaces/mediums 部分有值）
    # dry-run 只警告不阻断（先看 diff）；--apply 写盘前硬性拒绝
    bad = [(r, fs) for r in new_paints
           for fs in (missing_fields(r),) if fs and partial_data(r)]
    if bad:
        from collections import Counter
        by_field = Counter(f for _, fs in bad for f in fs)
        if args.apply:
            print(f"[error] input has {len(bad)} rows with partial data (some of "
                  f"color/bases/surfaces/mediums provided, merge rejected):")
            for f in REQUIRED_FIELDS:
                n = by_field.get(f, 0)
                if n:
                    samples = [f"{r.brand}:{r.code}" for r, fs in bad if f in fs][:3]
                    print(f"  - {f}: {n} rows  e.g. {', '.join(samples)}")
            print("  fix the source build.py (or provide all four fields) first")
            raise SystemExit(1)
        print(f"[warn] input has {len(bad)} rows with partial data (some of "
              f"color/bases/surfaces/mediums provided; --apply will reject):")
        for f in REQUIRED_FIELDS:
            n = by_field.get(f, 0)
            if n:
                samples = [f"{r.brand}:{r.code}" for r, fs in bad if f in fs][:3]
                print(f"  - {f}: {n} rows  e.g. {', '.join(samples)}")

    # 源 meta 注册：小宽表自带 sources + cli --source（默认同目录 source.json）覆盖/补充
    source_meta: dict = {}
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

    result = merge_wide(old, Wide(paints=new_paints), overwrite=not args.keep,
                        conflict=args.conflict)

    # 禁止冲突的 replace 键（extra）：conflict=error（默认）时双方都有值且不同 -> 直接报错退出
    # （dry-run 同样退出，强制显式选择 update/ignore；--apply 时未写盘）
    extra_conflicts = [c for c in result.report.conflicts
                       if len(c.split()) > 1 and c.split()[1] == "extra"]
    if args.conflict == "error" and extra_conflicts:
        print(f"[error] {len(extra_conflicts)} 处 extra 字段冲突（replace 键默认报错退出，"
              f"可用 --conflict update|ignore 指定行为）:")
        for c in extra_conflicts:
            print(f"  ! {c}")
        raise SystemExit(1)
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
    missing = sorted({s for row in new_paints for s in row.sources} - known)
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
                            "equivs": [e for e in r.equivs if (e.brand, e.code) not in drop]})
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
