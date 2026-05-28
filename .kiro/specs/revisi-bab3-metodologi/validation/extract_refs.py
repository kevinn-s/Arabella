#!/usr/bin/env python3
"""Extract all code references from Skripsi/bab3_metodologi.md.

A code reference is any backtick-quoted token that contains a path
(.rs / .toml / .frag / .vert / examples / tests / .cargo / assets) followed
by a colon and either a symbol or a line range or a single line number.
"""
import re
import sys

PATH = r"e:\Users\Documents\A - TUGAS BINUS\Thesis\Code\Skripsi\bab3_metodologi.md"

with open(PATH, encoding="utf-8") as fh:
    text = fh.read()

# Match backtick-delimited references of form `<file>:<rest>` where file
# is a path containing a slash and ending with .rs/.toml/.frag/.vert.
ref_re = re.compile(
    r"`([^`]*?\.(?:rs|toml|frag|vert))(?::([^`\s]+))?`",
    re.UNICODE,
)

refs = []
for m in ref_re.finditer(text):
    file_part = m.group(1)
    sym_part = m.group(2) or ""
    refs.append((file_part, sym_part))

# Deduplicate while preserving order.
seen = set()
unique = []
for r in refs:
    if r not in seen:
        seen.add(r)
        unique.append(r)

print(f"TOTAL backtick file mentions: {len(refs)}")
print(f"UNIQUE references          : {len(unique)}")
print()
print("== Unique references ==")
for f, s in unique:
    if s:
        print(f"{f}:{s}")
    else:
        print(f"{f}  (no symbol/line)")
