import csv
import json
import os

# 脚本所在目录（scripts/）与项目根目录
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(SCRIPT_DIR)

li = [['brand', 'serie', 'code', 'color', 'base', 'surface', 'medium']]

raw = {}

with open(os.path.join(SCRIPT_DIR, "gunze.csv"), "r", encoding='utf-8') as f:
    reader = csv.reader(f.readlines())
    next(reader)
    ja = raw.setdefault('ja', {})['gunze'] = {}
    en = raw.setdefault('en', {})['gunze'] = {}
    for base, serie, code, color, desc_ja, desc_en, surface, _, medium in reader:
        if desc_ja != "":
            ja[code] = desc_ja
        if desc_en != "":
            en[code] = desc_en
        li.append((
            'gunze',
            serie,
            code,
            int(color[1:], 16),
            1 << int(base),
            surface,
            medium,
        ))


with open(os.path.join(SCRIPT_DIR, "tamiya.csv"), "r", encoding='utf-8') as f:
    reader = csv.reader(f.readlines())
    next(reader)
    tamiya = []
    ja = raw.setdefault('ja', {})['tamiya'] = {}
    en = raw.setdefault('en', {})['tamiya'] = {}
    for color, serie, code, desc_ja, surface, base, medium in reader:
        ja[code] = desc_ja
        tamiya.append((
            'tamiya',
            serie,
            code,
            int(color[1:], 16),
            base,
            surface,
            medium,
        ))

    def f(x):
        s = x[2]
        if s[-1].isalpha():
            s = s[:-1]
        return int(s[len(x[1]):])

    tamiya.sort(key=f)
    li += tamiya


with open(os.path.join(SCRIPT_DIR, "ak.csv"), "r", encoding='utf-8') as f:
    reader = csv.reader(f.readlines())
    next(reader)
    ak = []
    es = raw.setdefault('es', {})['ak'] = {}
    en = raw.setdefault('en', {})['ak'] = {}
    for row in reader:
        code, serie, desc, color = row[0:4]
        code = code[2:]
        medium = row[-1]
        en[code] = desc
        ak.append((
            'ak',
            serie,
            code,
            int(color[1:], 16),
            1 << 3,
            'ME' if serie == 'M' else \
            'C' if desc.startswith('Clear ') else \
            'FL' if desc.startswith('Fluorescent ') else \
            'M',
            medium,
        ))

    ak.sort(key=lambda x:int(x[2][2:]))
    li += ak


with open(os.path.join(SCRIPT_DIR, "av.csv"), "r", encoding='utf-8') as f:
    reader = csv.reader(f.readlines())
    next(reader)
    av = []
    es = raw.setdefault('es', {})['av'] = {}
    en = raw.setdefault('en', {})['av'] = {}
    for surface, serie, ref, desc_en, desc_es, color, medium in reader:
        if desc_en != "":
            en[ref] = desc_en
        if desc_es != "":
            es[ref] = desc_es
        av.append((
            'av',
            serie,
            ref,
            int(color[1:], 16),
            1 << 1 if serie == 'LM' else 1 << 3,
            surface,
            medium,
        ))

    av.sort(key=lambda x:float(x[2]))
    li += av


with open(os.path.join(ROOT, "web/static/colors.csv"), "w", newline="", encoding="utf-8") as f:
    writer = csv.writer(f)
    writer.writerows(li)

with open(os.path.join(ROOT, "web/static/paints/raw.json"), "w", newline="", encoding="utf-8") as f:
    json.dump(raw, f, indent=2)
