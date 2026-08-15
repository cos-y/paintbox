"""pdfx CLI：python -m pdfx <source> [--out FILE] [--format json|csv] [--check-only]

可用源见 sources/。新源只需在 sources/ 下新增文件并暴露 `source` 实例。
输出格式由源自己的 serialize() 组织（JSON 默认；csv 仅对扁平记录生效）。
"""

from __future__ import annotations

import argparse
import csv
import importlib
import json
import sys
from pathlib import Path

from . import validate
from .core import Source

def _project_root() -> Path:
    """向上找含 web/ 目录的项目根（不依赖包嵌套层级）。"""
    return next(p for p in Path(__file__).resolve().parents if (p / "web").is_dir())

SOURCES_DIR = Path(__file__).parent / "sources"


def _load_source(name: str):
    # 支持文件路径（raw/<brand>/<source>/parse.py），也支持 pdfx.sources 内建名
    if name.endswith(".py") or "/" in name or "\\" in name:
        p = Path(name)
        if not p.is_absolute():
            p = _project_root() / p
        import importlib.util

        spec = importlib.util.spec_from_file_location(f"pdfx_ext_{p.stem}", p)
        if spec is None or spec.loader is None:
            print(f"cannot load source file: {p}", file=sys.stderr)
            raise SystemExit(1)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        return mod.source
    try:
        mod = importlib.import_module(f"pdfx.sources.{name}")
    except ImportError as e:
        print(f"unknown source '{name}'. available:", file=sys.stderr)
        for p in sorted(SOURCES_DIR.glob("*.py")):
            if p.name != "__init__.py":
                print(f"  - {p.stem}", file=sys.stderr)
        raise SystemExit(1) from e
    return mod.source


def _to_csv(records: list[dict]) -> str:
    keys: list[str] = []
    for r in records:
        for k in r:
            if k not in keys:
                keys.append(k)
    out = []
    w = csv.DictWriter(out, fieldnames=keys, extrasaction="ignore")
    w.writeheader()
    for r in records:
        w.writerow({k: v for k, v in r.items()})
    return "".join(out)


def main() -> None:
    ap = argparse.ArgumentParser(description="PDF layout extraction skeleton")
    ap.add_argument("source", help="source name (see sources/)")
    ap.add_argument("--out", help="output file path (relative to project root)")
    ap.add_argument("--format", choices=["json", "csv"], default="json",
                    help="serialization (only for flat records; source serialize() overrides)")
    ap.add_argument("--check-only", action="store_true", help="run validation and exit")
    args = ap.parse_args()

    src = _load_source(args.source)
    if not src.pdf:
        print(f"source '{src.name}' has no pdf configured")
        raise SystemExit(1)

    records = src.all_records()
    issues = src.validate(records)
    validate.report(records, issues)

    if not args.check_only and args.out:
        out = Path(args.out)
        if not out.is_absolute():
            out = _project_root() / out
        text = src.serialize(records) if src.serialize is not Source.serialize else (
            _to_csv(records) if args.format == "csv" else src.serialize(records)
        )
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(text, encoding="utf-8")
        print(f"written: {out}")
    raise SystemExit(1 if issues else 0)


if __name__ == "__main__":
    main()
