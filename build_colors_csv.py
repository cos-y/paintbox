import csv

li = [['brand', 'serie', 'code', 'color', 'desc', 'base', 'prop']]

with open("gunze.csv", "r", encoding='utf-8') as f:
    reader = csv.reader(f.readlines())
    next(reader)
    for base, serie, code, color, desc, prop, _ in reader:
        li.append((
            'gunze',
            serie,
            code,
            int(color[1:], 16),
            desc,
            1 << int(base),
            prop,
        ))


with open("tamiya.csv", "r", encoding='utf-8') as f:
    reader = csv.reader(f.readlines())
    next(reader)
    tamiya = []
    for color, serie, code, desc, prop, base in reader:
        tamiya.append((
            'tamiya',
            serie,
            code,
            int(color[1:], 16),
            desc,
            base,
            prop,
        ))

    def f(x):
        s = x[2]
        if s[-1].isalpha():
            s = s[:-1]
        return int(s[len(x[1]):])

    tamiya.sort(key=f)
    li += tamiya


with open("ak.csv", "r", encoding='utf-8') as f:
    reader = csv.reader(f.readlines())
    next(reader)
    for row in reader:
        code, serie, desc, color = row[0:4]
        li.append((
            'ak',
            serie,
            code,
            int(color[1:], 16),
            desc,
            1 << 3,
            'ME' if serie == 'M' else \
            'C' if desc.startswith('Clear ') else \
            'FL' if desc.startswith('Fluorescent ') else \
            'M',
        ))


with open("web/static/colors.csv", "w", newline="", encoding="utf-8") as f:
    writer = csv.writer(f)
    writer.writerows(li)
