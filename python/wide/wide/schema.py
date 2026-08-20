"""大宽表 schema（paintbox.wide.v2）。

设计约定：
- 只存"有来源的直接信息"：desc 只含源语言原文，翻译是阶段3 的派生数据，不进宽表
- 空字段 = 未知/未提供，merge 时不更新
- equivs 是发布方断言的等价，每条带 source（声明来源），不要求对称、不做传递闭包
- equiv 的 source 不并入行的 sources（行的 sources 只记非等价字段的来源）
- bases 位标志 merge 时 replace；surfaces/mediums 可 OR
- 每行 sources 引用 data/sources.json 的 meta（id → 描述）
"""

from __future__ import annotations

from typing import Any

from pydantic import BaseModel, ConfigDict, Field

SCHEMA = "paintbox.wide.v2"

_LANG_PAT = r"^[a-z]{2,3}$"


class Equiv(BaseModel):
    """等价声明：另一支油漆 + 该声明的来源（引用 sources registry 的 id）。"""

    model_config = ConfigDict(frozen=True)

    brand: str
    code: str
    source: str  # 谁断言了这个等价（ksp2026 / ak_equiv21 ...）


class Row(BaseModel):
    """一支油漆（宽表一行）。"""

    model_config = ConfigDict(frozen=True)

    brand: str
    serie: str | None = None
    code: str
    color: int | None = None  # 0xRRGGBB
    desc: dict[str, str] = Field(default_factory=dict)  # lang -> 源语言原文
    equivs: list[Equiv] = Field(default_factory=list)
    bases: int | None = None  # bitflags，merge 时 replace
    surfaces: int = 0  # bitflags，可 OR
    mediums: int = 0  # bitflags，可 OR
    sources: list[str] = Field(default_factory=list)  # 引用 sources.json 的 id
    note: str | None = None  # 人工维护说明，merge 不生成
    extra: dict[str, Any] | None = None  # 特殊漆料额外元信息（如透明度/类型），CSV 存 compact JSON，空 = 无

    def key(self) -> tuple[str, str]:
        return (self.brand, self.code)


class Wide(BaseModel):
    """宽表容器（data/wide.json 与各阶段小宽表共用）。"""

    model_config = ConfigDict(protected_namespaces=())

    schema_name: str = Field(default=SCHEMA, alias="schema", serialization_alias="schema")
    generatedAt: str = ""
    sources: dict[str, dict] = Field(default_factory=dict)  # 源 id -> source.json 内容
    paints: list[Row] = Field(default_factory=list)

    def by_key(self) -> dict[tuple[str, str], Row]:
        return {r.key(): r for r in self.paints}
