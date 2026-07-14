import re
import csv
import os


DIR = os.path.dirname(os.path.abspath(__file__))


def c_smali_to_csv(text: str):
  data = [["brand", "code", "desc", "rgb", "serie", "serie_code"]]

  r1 = r"\.method private final (\w+)\(Landroid/database/sqlite/SQLiteDatabase;\)V"
  r2 = r"invoke-static/range {v(\w+) \.\. v(\w+)}[^\n]*\n"
  r3 = r"const-string(?:/jumbo)? v(\w+), \"(.*)\""
  for m1 in re.finditer(r1, text):
    start = m1.end()
    end = text.index(".end method", start)
    s1 = text[start:end]
    i = 0
    for m2 in re.finditer(r2, s1):
      beg = m2.group(1)
      end = m2.group(2)
      s2 = s1[i:m2.start()]
      i = m2.end()
      li = []
      for m3 in re.finditer(r3, s2):
        li.append((int(m3.group(1)), m3.group(2)))
      li.sort(key=lambda x: x[0])
      li = [x for _, x in li]
      data.append((
        li[0].lower(), 
        li[1], 
        li[5], 
        li[2],
        li[3],
        int(li[6][1:], 16)
      ))

  csv_path = os.path.join(DIR, "web/static/colors.csv")
  with open(csv_path, "w", newline="", encoding="utf-8") as f:
    writer = csv.writer(f)
    writer.writerows(data)


with open("c.smali", "r") as f:
  text = f.read()
  c_smali_to_csv(text)

