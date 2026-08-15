"""纯结构校验（无任何数据依赖，框架与提取器都不感知项目数据）。

通用检查：记录键一致性、必需字段空值、记录数。
业务校验（如双向一致性）由各源在自己的 validate() 里实现。
"""

from __future__ import annotations

from typing import Iterable, Optional


def structural_issues(
    records: list[dict],
    *,
    required: Optional[Iterable[str]] = None,
    min_records: int = 0,
) -> list[str]:
    """纯结构问题列表：
    - 无记录（且设置了 min_records）
    - 记录键不一致（缺/多字段）
    - required 字段存在空值（None / '' / []）
    """
    issues: list[str] = []
    if not records:
        if min_records:
            issues.append(f"no records extracted (min {min_records})")
        return issues

    keys = set(records[0])
    for i, r in enumerate(records):
        if set(r) != keys:
            issues.append(f"record {i}: keys {sorted(set(r))} != {sorted(keys)}")
            break

    if required:
        req = list(required)
        for k in req:
            empty = [i for i, r in enumerate(records) if r.get(k) in (None, "", [])]
            if empty:
                issues.append(f"field '{k}' empty in {len(empty)}/{len(records)} records")
    return issues


def report(records: list[dict], issues: list[str]) -> None:
    print(f"records: {len(records)}")
    for issue in issues:
        print(f"[issue] {issue}")
    if not issues:
        print("[ok] structural checks passed")
