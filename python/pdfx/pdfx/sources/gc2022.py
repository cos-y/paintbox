"""gc2022 — Vallejo Game Color 2022 色卡目录（Colores-GC-2022.pdf）。

布局（横版 A4）：
- 40x26 填充色卡（右区 117 个）：色卡下方三行 = 色号(72.001) / 英文名 / 西语名
- 26x26 填充色卡（左区 96 个）：色号印在色卡内底部
- fill 颜色 = 色卡颜色

输出：每色卡一条记录 {code, name, name_es, hex}（26x26 卡无名称）。
"""

from __future__ import annotations

from ..core import Source, fill_hex, join, lines_of, load_words, region_words, swatches


class Gc2022(Source):
    name = "gc2022"
    pdf = ".claude/Colores-GC-2022.pdf"

    def records(self, page):
        words = load_words(page)
        # 色号印在色卡内底缘（inside），色名/西语名在下方（below）
        for card in swatches(page, kind="f", size=(40, 26), tol=3, words=words):
            below = region_words(words, card.rect, "below", pad=3)
            lines = lines_of(card.words + below, dy=2)
            if not lines:
                continue
            yield {
                "code": join(lines[0]),
                "name": join(lines[1], sep=" ") if len(lines) > 1 else None,
                "name_es": join(lines[2], sep=" ") if len(lines) > 2 else None,
                "hex": fill_hex(card.fill),
            }
        # 26x26 小色卡：色号印在卡底缘下方紧贴处
        for card in swatches(page, kind="f", size=(26, 26), tol=3, words=words):
            below = region_words(words, card.rect, "below", pad=3)
            lines = lines_of(card.words + below, dy=2)
            if not lines:
                continue
            yield {"code": join(lines[0]), "name": None, "name_es": None, "hex": fill_hex(card.fill)}


source = Gc2022()
