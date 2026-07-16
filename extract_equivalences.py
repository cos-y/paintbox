import csv
import re
import os

DIR = os.path.dirname(os.path.abspath(__file__))
R60_PATH = os.path.join(DIR, ".claude/com.pulgadas.hobbycolorconverter/smali/r60.smali")
UU0_PATH = os.path.join(DIR, ".claude/com.pulgadas.hobbycolorconverter/smali/uu0.smali")
COLORS_CSV = os.path.join(DIR, "web/static/colors.csv")
OUT_DIR = os.path.join(DIR, "web/static")

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
    with open(R60_PATH, "r", encoding="utf-8") as f:
        r60_text = f.read()
    with open(UU0_PATH, "r", encoding="utf-8") as f:
        uu0_text = f.read()

    r60_literals = extract_literals(r60_text)

    # 品牌数字id -> colors.csv里用的brand slug（复用Fabricantes表，字段：id,slug,name_es,name_en,desc_es,desc_en,url,date）
    id_to_brand = {}
    for lit in r60_literals:
        fields = parse_fields(lit)
        if fields and len(fields) == 8 and DATE_RE.match(fields[7]):
            id_, slug = fields[0], fields[1]
            id_to_brand[id_] = slug.lower()

    # Equivalences表：字段：id,brand_a,color_a,brand_b,color_b,provider,timestamp（provider这里用不上，丢弃）
    raw_rows = []
    for lit in extract_literals(uu0_text):
        fields = parse_fields(lit)
        if (
            fields
            and len(fields) == 7
            and DATE_RE.match(fields[6])
            and fields[1].isdigit()
            and fields[3].isdigit()
            and fields[5].isdigit()
        ):
            _id, brand_a, color_a, brand_b, color_b, _provider, _date = fields
            raw_rows.append((brand_a, color_a, brand_b, color_b))

    print(f"raw equivalence rows extracted: {len(raw_rows)}")

    # colors.csv里每一行的下标就是wasm里PaintEntry.index，直接按行号建立(brand,code)->index的映射
    with open(COLORS_CSV, encoding="utf-8") as f:
        r = csv.reader(f)
        next(r)
        paint_index = {(row[0], row[1].strip()): i for i, row in enumerate(r)}

    pairs = set()
    unresolved_brand_ids = set()
    unresolved_pairs = 0
    for brand_a, color_a, brand_b, color_b in raw_rows:
        slug_a = id_to_brand.get(brand_a)
        slug_b = id_to_brand.get(brand_b)
        if slug_a is None:
            unresolved_brand_ids.add(brand_a)
        if slug_b is None:
            unresolved_brand_ids.add(brand_b)
        if slug_a is None or slug_b is None:
            continue
        idx_a = paint_index.get((slug_a, color_a.strip()))
        idx_b = paint_index.get((slug_b, color_b.strip()))
        if idx_a is None or idx_b is None or idx_a == idx_b:
            unresolved_pairs += 1
            continue
        pairs.add((idx_a, idx_b) if idx_a < idx_b else (idx_b, idx_a))

    print(f"resolved index pairs (deduped): {len(pairs)}")
    print(f"dropped (unresolved paint on either side): {unresolved_pairs}")
    print(f"unresolved brand ids: {sorted(unresolved_brand_ids)}")

    with open(os.path.join(OUT_DIR, "equivalences.csv"), "w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        for a, b in sorted(pairs):
            w.writerow([a, b])


if __name__ == "__main__":
    main()
