"""akvallejo — AK 与 Vallejo 等价表（equiv_ak-vallejo.pdf）。

布局：表格，每行 = AK 色号 | 28x12 色块 | Vallejo 色号，页面多列组并行
（同一文本行内交替出现 AK码, 数字, AK码, 数字…）。

输出：每对 (AK码, Vallejo码) 一条记录 {ak, vallejo, hex}。
hex 取 AK 码与 Vallejo 码之间 x 范围内的色块填充色。
"""

from __future__ import annotations

import re

from ..core import Source, fill_hex, lines_of, load_words, swatches


class AkVallejo(Source):
    name = "akvallejo"
    pdf = ".claude/equiv_ak-vallejo.pdf"

    AK_RE = re.compile(r"^AK\d+$")
    NUM_RE = re.compile(r"^\d{2,4}$")

    def records(self, page):
        words = load_words(page)
        blocks = swatches(page, kind="f", size=(28, 12), tol=3)
        for line in lines_of(words, dy=6):
            # 行内 token 化：AK 码 / 数字 交替
            toks = []
            for w in line:
                if self.AK_RE.match(w.text):
                    toks.append(("ak", w))
                elif self.NUM_RE.match(w.text):
                    toks.append(("num", w))
            for i in range(len(toks) - 1):
                if toks[i][0] != "ak" or toks[i + 1][0] != "num":
                    continue
                ak, num = toks[i][1], toks[i + 1][1]
                # 色块：x 位于 AK 码与数字之间（且 y 与行对齐）
                hexc = None
                for b in blocks:
                    bcx = (b.rect.x0 + b.rect.x1) / 2
                    bcy = (b.rect.y0 + b.rect.y1) / 2
                    if ak.x1 <= bcx <= num.x0 and abs(bcy - ak.cy) < 8:
                        hexc = fill_hex(b.fill)
                        break
                yield {"ak": ak.text, "vallejo": num.text, "hex": hexc}


source = AkVallejo()
