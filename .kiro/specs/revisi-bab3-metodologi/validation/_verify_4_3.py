"""Phase 3 validator for Property 3 — Heading structure invariant."""
import sys
from pathlib import Path

TARGET = Path(r"e:\Users\Documents\A - TUGAS BINUS\Thesis\Code\Skripsi\bab3_metodologi.md")

REQUIRED = [
    ("# BAB 3 METODE PENELITIAN", 1),
    ("## 3.1 Diagram Alir Kerangka Berpikir", 2),
    ("## 3.2 Analisis Kebutuhan", 2),
    ("### 3.2.1 Analisis User", 3),
    ("### 3.2.2 Analisis Aplikasi Sejenis", 3),
    ("### 3.2.3 Rumusan dan Solusi Kebutuhan", 3),
    ("## 3.3 Perancangan Aplikasi", 2),
    ("### 3.3.1 Spesifikasi Aplikasi", 3),
    ("## 3.4 Perancangan Sistem", 2),
    ("### 3.4.1 Use Case Diagram", 3),
    ("### 3.4.2 Use Case Description", 3),
    ("### 3.4.3 Sequence Diagram", 3),
    ("### 3.4.4 Class Diagram", 3),
    ("## 3.5 Perancangan Algoritma", 2),
    ("## 3.6 Perancangan Layar", 2),
    ("## 3.7 Perancangan Database File", 2),
]

text = TARGET.read_text(encoding="utf-8")
lines = text.splitlines()

print(f"file_lines={len(lines)}")
print(f"line1={lines[0]!r}")
print(f"line1_pass={lines[0] == '# BAB 3 METODE PENELITIAN'}")

# Per-heading exact-text occurrence count (case-sensitive)
issues = []
positions = []  # (lineno, text, level)
for needle, level in REQUIRED:
    line_nos = [i + 1 for i, line in enumerate(lines) if line == needle]
    print(f"count[{needle!r}] = {len(line_nos)} at {line_nos}")
    if len(line_nos) != 1:
        issues.append(f"OCCURRENCE_FAIL: {needle!r} count={len(line_nos)}")
    else:
        positions.append((line_nos[0], needle, level))

# Check for stray headings at H1/H2/H3 not in required set (informational, doesn't fail unless violates invariant)
heading_lines = []
for i, line in enumerate(lines):
    s = line.lstrip()
    if line.startswith(("# ", "## ", "### ", "#### ")):
        heading_lines.append((i + 1, line))

required_set = {n for n, _ in REQUIRED}
stray = [(ln, t) for ln, t in heading_lines if t not in required_set]
print(f"stray_headings={stray}")

# Check ordering by line number
positions_sorted_by_canonical = positions  # in REQUIRED order
prev = -1
ordered_ok = True
for ln, t, _ in positions_sorted_by_canonical:
    if ln <= prev:
        ordered_ok = False
        issues.append(f"ORDER_FAIL: {t!r} at line {ln} not after previous {prev}")
    prev = ln
print(f"canonical_order_pass={ordered_ok}")

# H2 monotonic 3.1..3.7
h2_in_order = [(ln, t) for ln, t, lv in positions_sorted_by_canonical if lv == 2]
nums = []
for ln, t in h2_in_order:
    n = t.split(" ")[1]  # like '3.1'
    nums.append((ln, n))
print(f"h2_sequence={nums}")
expected_h2 = ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7"]
h2_pass = [n for _, n in nums] == expected_h2
print(f"h2_monotonic_pass={h2_pass}")

# H3 grouping under each H2: Y starts at 1 and ascends
from collections import defaultdict
h3_by_parent = defaultdict(list)
for ln, t, lv in positions_sorted_by_canonical:
    if lv == 3:
        # extract X.Y
        num = t.split(" ")[1]  # '3.2.1'
        parts = num.split(".")
        parent = f"{parts[0]}.{parts[1]}"
        y = int(parts[2])
        h3_by_parent[parent].append((ln, y))

# also need parent line numbers
parent_lines = {t.split(' ')[1]: ln for ln, t, lv in positions_sorted_by_canonical if lv == 2}
# next parent line
sorted_parents = sorted(parent_lines.items(), key=lambda kv: kv[1])
parent_ranges = {}
for i, (k, v) in enumerate(sorted_parents):
    next_v = sorted_parents[i+1][1] if i+1 < len(sorted_parents) else 10**9
    parent_ranges[k] = (v, next_v)

hierarchy_pass = True
for parent, children in h3_by_parent.items():
    p_start, p_end = parent_ranges[parent]
    ys = [y for _, y in children]
    expected_ys = list(range(1, len(ys) + 1))
    in_range = all(p_start < ln < p_end for ln, _ in children)
    if ys != expected_ys or not in_range:
        hierarchy_pass = False
        issues.append(f"HIERARCHY_FAIL: parent={parent} children={children} expected_ys={expected_ys} in_range={in_range}")
print(f"hierarchy_pass={hierarchy_pass}")

print("ISSUES:")
for x in issues:
    print(" -", x)
print("OVERALL_PASS=", not issues and lines[0] == '# BAB 3 METODE PENELITIAN' and ordered_ok and h2_pass and hierarchy_pass)
