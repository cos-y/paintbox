"""确定性 zh 翻译（基础版）：按 skill 固定翻译词表把英文名译为中文。

仅覆盖"确定性"部分——介质/效果/常用词按词表直译；无法确定性翻译的
（颜色专名、组合歧义）返回 None，由人工/AI 补充（阶段4 补翻译时用）。

规则（对应 skill「翻译的注意事项」）：
- 固定翻译：gloss->光泽, semi->半光, matt/flat->消光, metallic->金属,
  fluo->荧光, pearl->珠光
- 单字结果加修饰词（光泽黄/金属铜）
- 词序：英文形容词在前，中文保持"修饰词+名词"（Gloss Medium -> 光泽介质）
"""

from __future__ import annotations

import re

# 词表：英文词（小写）-> 中文。按"介质/效果/通用词"组织。
WORDS: dict[str, str] = {
    # 介质
    "medium": "介质", "varnish": "清漆", "thinner": "稀释剂",
    "putty": "补土", "mask": "遮盖", "glaze": "罩染",
    "retarder": "缓干", "crackle": "裂纹", "chipping": "掉漆",
    "improver": "流平剂", "softener": "软化剂", "fix": "固定剂",
    # 属性
    "gloss": "光泽", "glossy": "光泽", "semi": "半光", "satin": "半光",
    "matt": "消光", "flat": "消光", "metallic": "金属", "metal": "金属",
    "fluorescent": "荧光", "fluo": "荧光", "pearl": "珠光", "clear": "透明",
    # 修饰
    "plastic": "塑料", "liquid": "液体", "airbrush": "喷笔",
    "decal": "水贴", "flow": "流平",
}

# 固定短语优先匹配（整词序）
PHRASES: list[tuple[str, str]] = [
    ("airbrush thinner", "喷笔稀释剂"),
    ("flow improver", "流平剂"),
    ("decal softener", "水贴软化剂"),
    ("decal fix", "水贴固定剂"),
    ("plastic putty", "塑料补土"),
    ("liquid mask", "液体遮盖液"),
    ("gloss medium", "光泽介质"),
    ("matt medium", "消光介质"),
    ("metal medium", "金属介质"),
    ("glaze medium", "罩染介质"),
    ("retarder medium", "缓干介质"),
    ("crackle medium", "裂纹介质"),
    ("chipping medium", "掉漆介质"),
    ("gloss varnish", "光泽清漆"),
    ("matt varnish", "消光清漆"),
    ("satin varnish", "半光清漆"),
]


def translate_zh(en: str | None) -> str | None:
    """确定性英->中翻译；无法确定时返回 None。"""
    if not en:
        return None
    low = en.strip().lower()
    for phrase, zh in PHRASES:
        if low == phrase:
            return zh
    # 单词级：全部词可译才输出（避免半译）
    parts = re.split(r"[\s\-]+", low)
    out: list[str] = []
    for p in parts:
        if p in WORDS:
            out.append(WORDS[p])
        elif p in ("super",):
            out.append("超级")  # 品牌固定翻译：super->超级
        else:
            return None
    if not out:
        return None
    # 单字结果加修饰词：消光->消光色 等（skill：翻译结果只有一个字时加修饰）
    # 介质/非颜色词不加"色"字（如 消光添加剂）
    joined = "".join(out)
    return joined
