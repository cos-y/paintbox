"""共享推断：desc → surfaces / 系列默认四维值（bases/surfaces/mediums）。

供各源 aggregate.py 复用（skill「如何推导 surfaces 字段」的落地实现），
避免每个源一套推断规则导致漂移。

设计：
- infer_surfaces(desc)：按全部语言 desc 的 tag 词边界匹配，靠后的 flag
  优先（单 flag）；未命中返回 default（通常传系列默认 surfaces）。
- series_defaults(brand, serie)：从现有 data/wide.csv 归纳的同系列
  默认四维值（bases, surfaces, mediums），用于"缺数据时跟随现有"。
  未知系列返回 None，调用方自行决定。
- 系列默认归纳原则（2026-08-16, raw/av/cc070）：跟随现有同品牌同系列
  的多数值，merge 的 surfaces/mediums 是 union，新值取现有值的子集可
  保证 merge 后零变化。
"""

from __future__ import annotations

import re

# surfaces tag 表：flag 按 skill 列表顺序（靠后者优先命中）。
# 全部语言 desc 都会检查；tag 用词边界匹配，避免误伤颜色名形容词
# （如 \bgold\b 不命中 Golden/Silvergrey，silvergrey 是一个词）。
# 日文 tag（含非 ASCII）用子串匹配：\b 只认 ASCII 单词边界，对日文
# 完全失效（2026-08-17, raw/gaia：蛍光ブルー 等命中不到）；子串匹配
# 在 en/es 文本里不会误伤（日文词不会出现在拉丁文本中）。
SURFACE_TAGS: list[tuple[int, tuple[str, ...]]] = [
    (1, ("gloss", "glossy", "brillante")),                            # G
    (2, ("satin", "satinado", "semi", "セミグロス")),                  # SG
    (4, ("flat", "matt", "mate", "つや消し", "下地色", "フラット")),      # M
    (8, ("metal", "metallic", "metálico", "メタリック", "メタル",
         "gold", "silver", "bronze", "copper", "brass", "steel",
         "gunmetal", "oro", "plata", "cobre", "bronce", "acero",
         "シルバー", "ゴールド", "ブラス", "アイアン", "メッキ",
         "ミラー", "クローム")),                                       # ME
    (16, ("clear", "transparent", "transparente", "クリアー")),        # C
    (32, ("pearl", "nacar", "perla", "真珠", "パール", "プリズム")),     # PA
    (64, ("fluo", "fluorescent", "fluorescente", "蛍光")),             # FL
    (128, ("weathering", "wash")),                                            # W
]

# 系列默认 (bases, surfaces, mediums)，归纳自现有 data/wide.csv：
#   av MC/GC: bases=8（水基底）/ mediums=4；GC 默认半光 2
#   av GA:    bases=8 / mediums=5；默认半光 2
#   av LM:    bases=2（酒精基底）/ mediums=5；默认金属 8
SERIES_DEFAULTS: dict[tuple[str, str], tuple[int, int, int]] = {
    ("av", "MC"): (8, 4, 4),
    ("av", "GC"): (8, 2, 4),
    ("av", "GA"): (8, 2, 5),
    ("av", "LM"): (2, 8, 5),
}


def series_defaults(brand: str, serie: str) -> tuple[int, int, int] | None:
    """同系列默认 (bases, surfaces, mediums)；未知系列返回 None。"""
    return SERIES_DEFAULTS.get((brand, serie))


def infer_surfaces(desc: dict[str, str] | None, default: int = 0) -> int:
    """按 desc 推断 surfaces（单 flag，靠后的 tag 优先）。

    desc：语言 -> 名称文本（en/es/ja…全部检查）。
    default：未命中任何 tag 时的返回值（通常传系列默认 surfaces）。
    """
    text = " ".join((desc or {}).values()).lower()
    for flag, tags in reversed(SURFACE_TAGS):
        for t in tags:
            if t.isascii():
                if re.search(rf"\b{re.escape(t)}\b", text):
                    return flag
            else:
                if t in text:  # 日文：子串匹配（\b 对非 ASCII 失效）
                    return flag
    return default
