#!/usr/bin/env python3
"""For every numeric-line reference, dump line content so we can
visually confirm it matches what the doc claims it points to.
"""
import os
import re

ROOT = r"e:\Users\Documents\A - TUGAS BINUS\Thesis\Code"
DOC = os.path.join(ROOT, "Skripsi", "bab3_metodologi.md")

ref_re = re.compile(
    r"`([^`]*?\.(?:rs|toml|frag|vert)):([^`\s]+)`",
    re.UNICODE,
)
with open(DOC, encoding="utf-8") as fh:
    text = fh.read()

raw = ref_re.findall(text)
expanded = []
for path, spec in raw:
    if "," in spec:
        for part in [p.strip() for p in spec.split(",")]:
            expanded.append((path, part))
    else:
        expanded.append((path, spec))

# deduplicate, keep order
seen = set()
unique = []
for r in expanded:
    if r not in seen:
        seen.add(r)
        unique.append(r)

# Group by file
by_file = {}
for path, spec in unique:
    by_file.setdefault(path, []).append(spec)

for path in sorted(by_file):
    abs_path = os.path.join(ROOT, path.replace("/", os.sep))
    if not os.path.exists(abs_path):
        print(f"\n=== {path}  (FILE MISSING) ===")
        continue
    with open(abs_path, encoding="utf-8") as fh:
        lines = fh.readlines()
    print(f"\n=== {path}  (file has {len(lines)} lines) ===")
    for spec in by_file[path]:
        if re.fullmatch(r"\d+", spec):
            n = int(spec)
            content = lines[n - 1].rstrip() if 1 <= n <= len(lines) else "OUT_OF_RANGE"
            print(f"  L{n:>4}: {content[:140]}")
        elif re.fullmatch(r"\d+-\d+", spec):
            a, b = [int(x) for x in spec.split("-")]
            head = lines[a - 1].rstrip() if 1 <= a <= len(lines) else "OUT_OF_RANGE"
            tail = lines[b - 1].rstrip() if 1 <= b <= len(lines) else "OUT_OF_RANGE"
            print(f"  L{a}-{b}: head={head[:80]} | tail={tail[:80]}")
        else:
            print(f"  symbol :{spec}")
