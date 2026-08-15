"""pdfx core：PDF 布局提取通用原语。

设计原则：
- 提取器不感知任何项目数据（colors.csv / paints/*.json 一概不知）
- 每个源脚本只做两件事：
  1. 描述"这里有什么、怎么排版"（色卡尺寸/类型、文本区域、行/列结构）
  2. 把提取结果组织成自己的输出结构（dict 记录流，serialize 自定）
- 校验与提取解耦：框架只提供无数据依赖的纯结构校验，源可挂自定义校验

原语分类：
- 文本：load_words / region_words / lines_of / join
- 色卡：swatches（尺寸锚定/谓词）/ dominant_swatch_sizes / fill_hex
- 几何：series_codes（字母+数字合并）/ quadrants（分象限）
- 表格：column_bins / nearest
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable, Iterable, Optional

import fitz

# ---------------------------------------------------------------------------
# 数据模型
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Word:
    """带坐标的文本 token（get_text('words') 的一行）。"""

    x0: float
    y0: float
    x1: float
    y1: float
    text: str

    @property
    def cx(self) -> float:
        return (self.x0 + self.x1) / 2

    @property
    def cy(self) -> float:
        return (self.y0 + self.y1) / 2


@dataclass(frozen=True)
class Code:
    """几何合并后的 系列字母+数字 码（如 'C137'）。

    ax/ay 是锚点坐标，用于位置分类。锚点必须用"字母自身"的坐标：
    若用被合并的复合 word（如 '（フィールドグレー1）32'）的 x0，其可能
    在色卡左侧，会把码带偏到错误的象限。
    """

    text: str
    ax: float
    ay: float


@dataclass
class Swatch:
    """检测到的色卡：矩形 + 填充色 + 卡内文本。"""

    rect: fitz.Rect
    fill: Optional[tuple] = None
    words: list[Word] = field(default_factory=list)


# ---------------------------------------------------------------------------
# 文本
# ---------------------------------------------------------------------------


def _project_root() -> Path:
    """向上找含 web/ 目录的项目根（不依赖包嵌套层级）。"""
    return next(p for p in Path(__file__).resolve().parents if (p / "web").is_dir())

def load_words(page: fitz.Page, clip: Optional[fitz.Rect] = None) -> list[Word]:
    """加载页面（或矩形区域）内的文本 token，丢弃空串。"""
    words = []
    for w in page.get_text("words", clip=clip):
        text = w[4].strip()
        if text:
            words.append(Word(w[0], w[1], w[2], w[3], text))
    return words


def region_words(words: list[Word], rect: fitz.Rect, region: str, pad: float = 0) -> list[Word]:
    """取矩形某区域内的 words（按 word 中心点判断）。

    region: 'inside' | 'above' | 'below' | 'left' | 'right'
    - inside：矩形内
    - below/above：紧贴矩形下方/上方，高度取矩形自身高度（至少 30pt）
    - left/right：紧贴左侧/右侧，宽度取矩形自身宽度（至少 40pt）
    pad：区域向外扩展的边距
    """
    h = max(rect.height, 30)
    w = max(rect.width, 40)
    if region == "inside":
        r = fitz.Rect(rect.x0 - pad, rect.y0 - pad, rect.x1 + pad, rect.y1 + pad)
    elif region == "below":
        r = fitz.Rect(rect.x0 - pad, rect.y1, rect.x1 + pad, rect.y1 + h + pad)
    elif region == "above":
        r = fitz.Rect(rect.x0 - pad, rect.y0 - h - pad, rect.x1 + pad, rect.y0 + pad)
    elif region == "left":
        r = fitz.Rect(rect.x0 - w - pad, rect.y0 - pad, rect.x0 + pad, rect.y1 + pad)
    elif region == "right":
        r = fitz.Rect(rect.x1 - pad, rect.y0 - pad, rect.x1 + w + pad, rect.y1 + pad)
    else:
        raise ValueError(f"unknown region: {region}")
    return [wd for wd in words if r.x0 <= wd.cx <= r.x1 and r.y0 <= wd.cy <= r.y1]


def lines_of(words: list[Word], dy: float = 2.0) -> list[list[Word]]:
    """按 y 坐标聚类成行（同一行 y0 间距 <= dy），行内按 x 排序。

    用于：色卡下方多行文本（色号行、色名行…）、表格行分组。
    """
    if not words:
        return []
    ws = sorted(words, key=lambda w: (w.y0, w.x0))
    lines: list[list[Word]] = [[ws[0]]]
    for w in ws[1:]:
        if w.y0 - lines[-1][-1].y0 <= dy:
            lines[-1].append(w)
        else:
            lines.append([w])
    return [sorted(ln, key=lambda w: w.x0) for ln in lines]


def join(words: list[Word], sep: str = "") -> str:
    return sep.join(w.text for w in words)


# ---------------------------------------------------------------------------
# 色卡检测
# ---------------------------------------------------------------------------


def swatches(
    page: fitz.Page,
    *,
    kind: str = "fs",
    size: Optional[tuple[float, float]] = None,
    tol: float = 5,
    predicate: Optional[Callable[[dict, fitz.Rect], bool]] = None,
    words: Optional[list[Word]] = None,
) -> list[Swatch]:
    """检测页面色卡矩形，返回 Swatch（含 fill 与卡内 words）。

    - kind：'s' 描边矩形 / 'f' 或 'fs' 填充矩形
    - size：固定尺寸锚定 (w, h)，容差 tol
    - predicate：自由判定（替代 kind/size）
    - words：全页 words（load_words 一次取好），传入后按中心点绑定到各色卡
    """
    out: list[Swatch] = []
    for d in page.get_drawings():
        r = d["rect"]
        if predicate is not None:
            if not predicate(d, r):
                continue
        else:
            if d["type"] != kind:
                continue
            if size and (abs(r.width - size[0]) > tol or abs(r.height - size[1]) > tol):
                continue
        out.append(Swatch(rect=r, fill=d.get("fill")))
    # 按坐标去重（get_drawings 可能重复返回同一矩形）
    seen = set()
    uniq = []
    for s in out:
        key = (round(s.rect.x0, 1), round(s.rect.y0, 1), round(s.rect.x1, 1), round(s.rect.y1, 1))
        if key not in seen:
            seen.add(key)
            uniq.append(s)
    if words:
        for s in uniq:
            s.words = [
                wd for wd in words
                if s.rect.x0 <= wd.cx <= s.rect.x1 and s.rect.y0 <= wd.cy <= s.rect.y1
            ]
    return uniq


def dominant_swatch_sizes(
    drawings: list[dict], *, min_area: float = 300, top_n: int = 4
) -> list[tuple[float, float]]:
    """统计最常见的填充矩形尺寸（用于未知 PDF 的色卡尺寸探测）。"""
    from collections import Counter

    sizes: Counter = Counter()
    for d in drawings:
        r = d.get("rect")
        if not r or d.get("type") not in ("f", "fs") or d.get("fill") is None:
            continue
        if r.width * r.height < min_area:
            continue
        sizes[(round(r.width), round(r.height))] += 1
    return [k for k, _ in sizes.most_common(top_n)]


def fill_hex(fill: Optional[tuple]) -> Optional[str]:
    """填充色 (0-1 float 三元组) → '#RRGGBB'。无填充返回 None。"""
    if not fill:
        return None
    rgb = fill[:3]
    return "#{:02X}{:02X}{:02X}".format(*(round(c * 255) for c in rgb))


# ---------------------------------------------------------------------------
# 几何：系列码合并 + 象限
# ---------------------------------------------------------------------------


def series_codes(
    words: list[Word],
    *,
    series_re: str = r"[A-Z]{1,2}",
    num_re: str = r"\d{1,3}",
    full_re: str = r"[A-Z]{1,4}\d{1,3}[A-Z]?$",
    max_dy: float = 4.0,
    max_gap: float = 4.0,
    composite: bool = True,
) -> list[Code]:
    """把色号类文本合并为完整码。三类输入：

    1. 完整码 word（如 'CMC11'）：直接产出，锚点 = word 中心
    2. 系列字母 word + 独立数字 word（'H' + '77' → 'H77'）：
       文本层常拆成两个词且属于不同 block/行，只能按几何配对：
       数字在字母右侧紧邻、y 中心接近。
    3. 复合 word 尾部数字（'（フィールドグレー1）32' 粘成一个词）：
       只在 composite=True 时启用。注意它可能误抓色名尾数
       （如色名 'ハイライト1' 会被提取出 '1'），所以仅在明确
       需要替换标注的区域（如色卡右下角）开启。

    max_dy 必须按行高收紧（约行高一半），否则会把相邻行的字符合并错
    （踩过：'C81' 下方 'H33' 行被错并成 'C33'）。
    """
    series_pat = re.compile(series_re)
    num_pat = re.compile(num_re)
    full_pat = re.compile(full_re)
    codes: list[Code] = []
    for w in words:
        # 1. 完整码 word（跳过纯字母 word，避免重复处理）
        if full_pat.fullmatch(w.text) and not series_pat.fullmatch(w.text):
            codes.append(Code(w.text, w.cx, w.cy))
            continue
        if not series_pat.fullmatch(w.text):
            continue
        # 2. 优先：独立数字 word
        cand: Optional[Word] = None
        for w2 in words:
            if not num_pat.fullmatch(w2.text):
                continue
            if w2.x0 >= w.x1 - 1 and w2.x0 <= w.x1 + max_gap and abs(w.cy - w2.cy) < max_dy:
                if cand is None or w2.x0 < cand.x0:
                    cand = w2
        if cand is not None:
            codes.append(Code(w.text + cand.text, w.x0, w.cy))
            continue
        # 3. 兑底：复合 word 尾部数字（仅 composite=True）
        if not composite:
            continue
        for w2 in words:
            m = re.search(r"(\d+)$", w2.text)
            if not m:
                continue
            if w2.x0 <= w.x1 + max_gap and w2.x1 >= w.x1 - 1 and abs(w.cy - w2.cy) < max_dy:
                codes.append(Code(w.text + m.group(1), w.x0, w.cy))
                break
    return codes


def quadrants(rect: fitz.Rect, codes: list[Code]) -> dict[str, list[Code]]:
    """把码按矩形中线分成 tl/tr/bl/br 四个象限（位置语义由源自行声明）。"""
    cx, cy = (rect.x0 + rect.x1) / 2, (rect.y0 + rect.y1) / 2
    q: dict[str, list[Code]] = {"tl": [], "tr": [], "bl": [], "br": []}
    for c in codes:
        q[("t" if c.ay < cy else "b") + ("l" if c.ax < cx else "r")].append(c)
    return q


# ---------------------------------------------------------------------------
# 表格：列分箱
# ---------------------------------------------------------------------------


def column_bins(words: list[Word], gap: float = 12.0) -> list[float]:
    """把 words 的 x 中心聚成列（相邻聚类间隙 > gap），返回各列中心。"""
    xs = sorted(w.cx for w in words)
    groups: list[list[float]] = []
    for x in xs:
        if groups and x - groups[-1][-1] <= gap:
            groups[-1].append(x)
        else:
            groups.append([x])
    return [sum(g) / len(g) for g in groups]


def nearest(bins: list[float], x: float) -> int:
    """返回 x 最近的列下标。"""
    return min(range(len(bins)), key=lambda i: abs(bins[i] - x))


# ---------------------------------------------------------------------------
# Source 基类
# ---------------------------------------------------------------------------


class Source:
    """PDF 提取源。子类职责：

    - records(page) -> Iterable[dict]：描述布局并产出记录
    - serialize(records) -> str：组织输出格式（默认 JSON 记录列表，可覆盖为 dict/csv/…）
    - validate(records) -> list[str]：可选的自定义校验（框架只做纯结构校验）
    """

    name = ""
    pdf = ""

    def records(self, page: fitz.Page) -> Iterable[dict]:
        raise NotImplementedError

    def all_records(self) -> list[dict]:
        out: list[dict] = []
        with fitz.open(self.pdf_path) as doc:
            for page in doc:
                out.extend(self.records(page))
        return out

    def serialize(self, records: list[dict]) -> str:
        return json.dumps(records, ensure_ascii=False, indent=1)

    def validate(self, records: list[dict]) -> list[str]:
        from .validate import structural_issues

        return structural_issues(records)

    @property
    def pdf_path(self) -> Path:
        root = _project_root()
        p = Path(self.pdf)
        return p if p.is_absolute() else root / p
