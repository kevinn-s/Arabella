#!/usr/bin/env python3
"""Verify every code reference inside Skripsi/bab3_metodologi.md.

For each reference of the form `<path>:<spec>`:
    * Verify <path> exists.
    * If <spec> is a single integer N, verify file has >= N lines AND
      print the line content for human inspection.
    * If <spec> is N-M, verify N <= M and file has >= M lines AND
      print first/last line content.
    * If <spec> is a symbol (non-numeric token), search the file for a
      definition of that symbol and report the line number where it was
      found (or NOT FOUND).
"""
import os
import re
import sys

ROOT = r"e:\Users\Documents\A - TUGAS BINUS\Thesis\Code"
DOC = os.path.join(ROOT, "Skripsi", "bab3_metodologi.md")

# Find references of form `path:spec` inside backticks.
# path is anything ending in .rs/.toml/.frag/.vert
ref_re = re.compile(
    r"`([^`]*?\.(?:rs|toml|frag|vert)):([^`\s]+)`",
    re.UNICODE,
)

with open(DOC, encoding="utf-8") as fh:
    text = fh.read()

raw_refs = ref_re.findall(text)
# Some references have multi-line forms like "534, 539, 544, 549, 554, 559"
# captured as one piece up to the closing backtick. Expand them.
expanded = []
for path, spec in raw_refs:
    # Comma-separated line numbers: split each.
    if "," in spec:
        parts = [p.strip() for p in spec.split(",")]
        for part in parts:
            expanded.append((path, part))
    else:
        expanded.append((path, spec))

# Deduplicate.
seen = set()
unique_refs = []
for r in expanded:
    if r not in seen:
        seen.add(r)
        unique_refs.append(r)


def read_file_lines(rel_path):
    abs_path = os.path.join(ROOT, rel_path.replace("/", os.sep))
    if not os.path.exists(abs_path):
        return None
    with open(abs_path, encoding="utf-8") as fh:
        return fh.readlines()


def find_symbol(lines, sym):
    """Look for definition of `sym` in source lines.

    Accepts patterns like 'fn sym', 'struct sym', 'enum sym',
    'const sym', 'pub fn sym', 'impl Type {', 'impl Type::sym',
    or 'sym(' for the GLSL shader.
    """
    if "::" in sym:
        # Method like Blocks::build_block — look for fn build_block.
        type_name, method_name = sym.split("::", 1)
        m_re = re.compile(
            rf"\b(?:pub\s+(?:\(crate\)\s+)?)?(?:async\s+)?fn\s+{re.escape(method_name)}\b"
        )
        for i, line in enumerate(lines, 1):
            if m_re.search(line):
                return i, line.rstrip()
    else:
        m_re = re.compile(
            rf"\b(?:pub\s+(?:\(crate\)\s+)?)?(?:fn|struct|enum|trait|const|static|type)\s+{re.escape(sym)}\b"
        )
        for i, line in enumerate(lines, 1):
            if m_re.search(line):
                return i, line.rstrip()
        # GLSL fragment shader functions: type returntype, void name(...).
        glsl_re = re.compile(rf"\b{re.escape(sym)}\s*\(")
        for i, line in enumerate(lines, 1):
            if glsl_re.search(line):
                return i, line.rstrip()
    return None, None


total = 0
ok = 0
broken = []

for path, spec in unique_refs:
    total += 1
    lines = read_file_lines(path)
    if lines is None:
        broken.append((path, spec, "file does not exist"))
        continue

    # Numeric single line.
    if re.fullmatch(r"\d+", spec):
        n = int(spec)
        if n < 1 or n > len(lines):
            broken.append((path, spec, f"line {n} out of file range (file has {len(lines)} lines)"))
            continue
        ok += 1
        continue

    # Numeric range.
    m = re.fullmatch(r"(\d+)-(\d+)", spec)
    if m:
        a, b = int(m.group(1)), int(m.group(2))
        if a > b:
            broken.append((path, spec, f"start ({a}) > end ({b})"))
            continue
        if b > len(lines) or a < 1:
            broken.append((path, spec, f"range out of file (file has {len(lines)} lines)"))
            continue
        ok += 1
        continue

    # Symbol reference.
    found_line, found_text = find_symbol(lines, spec)
    if found_line is None:
        broken.append((path, spec, "symbol not found"))
        continue
    ok += 1

print(f"TOTAL unique references: {total}")
print(f"VALID                  : {ok}")
print(f"BROKEN                 : {len(broken)}")
print()
if broken:
    print("== BROKEN ==")
    for path, spec, reason in broken:
        print(f"  {path}:{spec}  -- {reason}")
else:
    print("All references validated.")
