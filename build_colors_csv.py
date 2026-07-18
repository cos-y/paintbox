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
            base,
            prop,
        ))


with open("web/static/colors.csv", "w", newline="", encoding="utf-8") as f:
    writer = csv.writer(f)
    writer.writerows(li)
