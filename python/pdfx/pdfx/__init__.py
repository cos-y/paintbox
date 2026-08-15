"""pdfx — PDF 布局提取通用骨架（基于 PyMuPDF）。

用法：cd scripts && ../venv/Scripts/python.exe -m pdfx <source> [--out FILE]
每新增一个 PDF 源：在 sources/ 下新建文件，描述布局（swatches/region_words/
lines_of…），定义 records() 输出记录、serialize() 组织格式、validate() 自定义校验。
"""

from .core import (
    Code,
    Source,
    Swatch,
    Word,
    column_bins,
    dominant_swatch_sizes,
    fill_hex,
    join,
    lines_of,
    load_words,
    nearest,
    quadrants,
    region_words,
    series_codes,
    swatches,
)

__all__ = [
    "Code",
    "Source",
    "Swatch",
    "Word",
    "column_bins",
    "dominant_swatch_sizes",
    "fill_hex",
    "join",
    "lines_of",
    "load_words",
    "nearest",
    "quadrants",
    "region_words",
    "series_codes",
    "swatches",
]
