import csv
import re
import json
import os
import shutil

DIR = os.path.dirname(os.path.abspath(__file__))
SMALI_PATH = os.path.join(DIR, ".claude/com.pulgadas.hobbycolorconverter/smali/r60.smali")
DRAWABLE_DIR = os.path.join(DIR, ".claude/com.pulgadas.hobbycolorconverter/res/drawable")
COLORS_CSV = os.path.join(DIR, "web/static/colors.csv")
THUMBS_OUT = os.path.join(DIR, "web/static/series-thumbs")

DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")


def unescape(s: str) -> str:
    def repl_u(m):
        return chr(int(m.group(1), 16))

    s = re.sub(r"\\u([0-9a-fA-F]{4})", repl_u, s)
    s = s.replace("\\'", "'").replace('\\"', '"')
    return s


def extract_literals(text: str):
    literals = []
    for m in re.finditer(r'const-string(?:/jumbo)?\s+\S+,\s*"((?:[^"\\]|\\.)*)"', text):
        literals.append(unescape(m.group(1)))
    return literals


def parse_fields(literal: str):
    if not literal.startswith("'"):
        return None
    return re.findall(r"'((?:[^'\\]|\\.)*)'", literal)


def main():
    with open(SMALI_PATH, "r", encoding="utf-8") as f:
        text = f.read()

    literals = extract_literals(text)

    brands = {}
    series = {}

    for lit in literals:
        fields = parse_fields(lit)
        if not fields:
            continue

        if len(fields) == 8 and DATE_RE.match(fields[7]):
            id_, slug, name_es, name_en, desc_es, desc_en, url, date = fields
            key = slug.lower()
            brands[key] = {
                "slug": slug,
                "name": name_en,
                "desc": desc_en,
                "url": url,
            }
        elif len(fields) == 7 and DATE_RE.match(fields[6]):
            id_, brand_slug, name_es, desc_es, name_en, desc_en, date = fields
            key = (brand_slug.lower(), id_)
            series[f"{key[0]}::{key[1]}"] = {
                "brand": brand_slug.lower(),
                "serie": id_,
                "name": name_en,
                "desc": desc_en,
            }

    # 只保留 colors.csv 里实际用到的 brand / (brand, serie) 组合
    with open(COLORS_CSV, encoding="utf-8") as f:
        r = csv.reader(f)
        next(r)  # 跳过表头（表头文字和实际列含义是错位的，按位置读取）
        rows = list(r)
    used_brands = {row[0] for row in rows}
    used_series = {(row[0], row[3]) for row in rows}

    brands = {k: v for k, v in brands.items() if k in used_brands}
    series = {k: v for k, v in series.items() if (v["brand"], v["serie"]) in used_series}

    print(f"brands kept: {len(brands)} / used_brands: {len(used_brands)}")
    print(f"series kept: {len(series)} / used_series: {len(used_series)}")
    print("brands missing metadata:", used_brands - set(brands.keys()))
    print("series missing metadata:", used_series - {(v["brand"], v["serie"]) for v in series.values()})

    out_dir = os.path.join(DIR, "web/static")
    with open(os.path.join(out_dir, "brands.json"), "w", encoding="utf-8") as f:
        json.dump(list(brands.values()), f, ensure_ascii=False, indent=2)
    with open(os.path.join(out_dir, "series.json"), "w", encoding="utf-8") as f:
        json.dump(list(series.values()), f, ensure_ascii=False, indent=2)

    # 拷贝系列缩略图（大小写不敏感匹配 res/drawable 里的文件名）
    drawable_files = {f.lower(): f for f in os.listdir(DRAWABLE_DIR)}
    os.makedirs(THUMBS_OUT, exist_ok=True)
    copied = 0
    missing_thumbs = []
    for v in series.values():
        src_name = f"{v['brand']}_gama_{v['serie']}.jpg".lower()
        if src_name in drawable_files:
            shutil.copyfile(
                os.path.join(DRAWABLE_DIR, drawable_files[src_name]),
                os.path.join(THUMBS_OUT, f"{v['brand']}-{v['serie']}.jpg"),
            )
            copied += 1
        else:
            missing_thumbs.append(src_name)
    print(f"thumbnails copied: {copied} / {len(series)}")
    print("missing thumbnails:", missing_thumbs)


if __name__ == "__main__":
    main()
