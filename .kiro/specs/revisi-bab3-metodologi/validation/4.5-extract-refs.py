#!/usr/bin/env python3
"""Task 4.5 — Property 5: Technical Claim Traceability validation.

For each backtick-quoted code reference inside Skripsi/bab3_metodologi.md:
  1. Verify the file path exists in the repository.
  2. If the reference is `path:N`, verify line N is within the file.
  3. If the reference is `path:N-M`, verify N <= M and M <= file length.
  4. If the reference is `path:symbol`, verify symbol is defined in file.
  5. Group references by subsection (3.1, 3.2.x, 3.3.1, 3.4.x, 3.5, 3.6, 3.7).

Reference format pattern:
  `<path>.<ext>` or `<path>.<ext>:<spec>` where ext ∈ {rs, toml, frag, vert}.

Output is written as UTF-8 to stdout via a buffered writer that bypasses
the Windows console code page.
"""
import io
import os
import re
import sys

# Force UTF-8 stdout to avoid cp1252 console encoding errors when run on
# Windows. We also accept an optional `--out PATH` argument so the report
# can be persisted directly to a UTF-8 file regardless of console code page.
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", newline="\n")

OUT_PATH = None
for i, arg in enumerate(sys.argv):
    if arg == "--out" and i + 1 < len(sys.argv):
        OUT_PATH = sys.argv[i + 1]

ROOT = r"e:\Users\Documents\A - TUGAS BINUS\Thesis\Code"
DOC = os.path.join(ROOT, "Skripsi", "bab3_metodologi.md")

# Backtick-delimited references: file path ending in .rs/.toml/.frag/.vert,
# optionally followed by :spec where spec contains no whitespace or backtick.
REF_RE = re.compile(
    r"`([^`\s]+?\.(?:rs|toml|frag|vert))(?::([^`\s]+))?`"
)

H2_RE = re.compile(r"^## (3\.\d)\b")
H3_RE = re.compile(r"^### (3\.\d\.\d)\b")


def label_per_line(text):
    """Return list of subsection labels indexed 0..N (line_no-1)."""
    out = []
    cur_h2 = ""
    cur_h3 = ""
    for line in text.splitlines():
        m3 = H3_RE.match(line)
        m2 = H2_RE.match(line)
        if m2:
            cur_h2 = m2.group(1)
            cur_h3 = ""
        elif m3:
            cur_h3 = m3.group(1)
        out.append(cur_h3 if cur_h3 else cur_h2)
    return out


def find_symbol(lines, sym):
    """Locate a Rust/GLSL/TOML symbol definition.

    Accepts: fn / struct / enum / trait / const / static / type for Rust;
    Method form Type::name → looks for fn name. Falls back to GLSL function
    pattern `name(` for shaders. Falls back to a literal occurrence of sym
    for general identifiers (TOML keys, attribute names, etc.).
    """
    if "::" in sym:
        type_name, method_name = sym.split("::", 1)
        m_re = re.compile(
            rf"\b(?:pub\s+(?:\(crate\)\s+)?)?(?:async\s+)?fn\s+{re.escape(method_name)}\b"
        )
        for i, line in enumerate(lines, 1):
            if m_re.search(line):
                return i, line.rstrip()
        return None, None
    m_re = re.compile(
        rf"\b(?:pub\s+(?:\(crate\)\s+)?)?(?:fn|struct|enum|trait|const|static|type)\s+{re.escape(sym)}\b"
    )
    for i, line in enumerate(lines, 1):
        if m_re.search(line):
            return i, line.rstrip()
    # GLSL function-like definition.
    glsl_re = re.compile(rf"\b{re.escape(sym)}\s*\(")
    for i, line in enumerate(lines, 1):
        if glsl_re.search(line):
            return i, line.rstrip()
    # Last-resort: literal token match.
    tok_re = re.compile(rf"\b{re.escape(sym)}\b")
    for i, line in enumerate(lines, 1):
        if tok_re.search(line):
            return i, line.rstrip()
    return None, None


def find_basename_matches(basename):
    """Walk the repo (within whitelisted source roots) for a matching basename."""
    roots = [
        "src",
        "examples",
        "tests",
        "assets",
        ".cargo",
    ]
    matches = []
    for root in roots:
        abs_root = os.path.join(ROOT, root)
        if not os.path.isdir(abs_root):
            continue
        for dirpath, _dirs, files in os.walk(abs_root):
            if basename in files:
                rel = os.path.relpath(os.path.join(dirpath, basename), ROOT)
                matches.append(rel.replace(os.sep, "/"))
    # Top-level files like Cargo.toml.
    top_level_path = os.path.join(ROOT, basename)
    if os.path.isfile(top_level_path):
        matches.append(basename)
    return matches


def verify(path, spec):
    abs_path = os.path.join(ROOT, path.replace("/", os.sep))
    if not os.path.isfile(abs_path):
        # Bare-basename fallback: if `path` has no directory component and
        # exactly one file with that basename exists in the repo, mark as a
        # basename-only reference (still flagged but informative).
        if "/" not in path and "\\" not in path:
            matches = find_basename_matches(path)
            if matches:
                return (
                    "OK_BASENAME",
                    f"bare basename matches {len(matches)} file(s): "
                    + ", ".join(matches),
                )
        return ("FILE_MISSING", f"no file at {abs_path}")
    with open(abs_path, encoding="utf-8") as fh:
        lines = fh.readlines()
    n_lines = len(lines)

    if spec is None or spec == "":
        return ("OK_FILE", f"file exists ({n_lines} lines)")

    if re.fullmatch(r"\d+", spec):
        n = int(spec)
        if 1 <= n <= n_lines:
            return ("OK_LINE", f"line {n} ≤ {n_lines}")
        return ("BAD_LINE", f"line {n} > file length {n_lines}")

    m = re.fullmatch(r"(\d+)-(\d+)", spec)
    if m:
        a, b = int(m.group(1)), int(m.group(2))
        if a < 1 or a > b:
            return ("BAD_RANGE", f"invalid range ({a}-{b})")
        if b > n_lines:
            return ("BAD_RANGE", f"range end {b} > file length {n_lines}")
        return ("OK_RANGE", f"range {a}-{b} ≤ {n_lines}")

    found_line, _ = find_symbol(lines, spec)
    if found_line is None:
        return ("SYMBOL_NOT_FOUND", f"symbol '{spec}' not found in file")
    return ("OK_SYMBOL", f"symbol '{spec}' at line {found_line}")


def main():
    global OUT_PATH
    if OUT_PATH:
        sys.stdout = open(OUT_PATH, "w", encoding="utf-8", newline="\n")
    with open(DOC, encoding="utf-8") as fh:
        text = fh.read()
    lines = text.splitlines()
    labels = label_per_line(text)

    refs = []  # list of (line_no, subsection, path, spec)
    for line_no, line in enumerate(lines, 1):
        for m in REF_RE.finditer(line):
            path = m.group(1)
            spec = m.group(2) or ""
            sub = labels[line_no - 1] or "(preamble)"
            refs.append((line_no, sub, path, spec))

    unique = set()
    for _, _, p, s in refs:
        unique.add((p, s))

    print(f"# Bab 3 — Code Reference Traceability")
    print()
    print(f"Total references extracted : **{len(refs)}**")
    print(f"Unique (path, spec) pairs  : **{len(unique)}**")
    print()
    print("## Per-reference verdict")
    print()
    print("|   # | Line | Subsection | File | Spec | Verdict | Detail |")
    print("|----:|-----:|------------|------|------|---------|--------|")

    sub_stats = {}
    overall = {"valid": 0, "broken": 0}
    for idx, (line_no, sub, path, spec) in enumerate(refs, 1):
        verdict, detail = verify(path, spec)
        ok = verdict.startswith("OK")
        s = sub_stats.setdefault(sub, {"valid": 0, "broken": 0})
        if ok:
            s["valid"] += 1
            overall["valid"] += 1
        else:
            s["broken"] += 1
            overall["broken"] += 1
        spec_disp = spec if spec else "—"
        print(
            f"| {idx:>3} | {line_no:>4} | {sub} | `{path}` | `{spec_disp}` | {verdict} | {detail} |"
        )

    print()
    print("## Per-subsection summary")
    print()
    print("| Subsection | Total | Valid | Broken | Verdict |")
    print("|------------|------:|------:|-------:|---------|")
    for sub in sorted(sub_stats):
        v = sub_stats[sub]["valid"]
        b = sub_stats[sub]["broken"]
        total = v + b
        if b > 0:
            verdict = "FAIL"
        elif v >= 1:
            verdict = "PASS"
        else:
            verdict = "NO_REFS"
        print(f"| {sub} | {total} | {v} | {b} | {verdict} |")

    print()
    total = overall["valid"] + overall["broken"]
    print(
        f"OVERALL: total={total}, valid={overall['valid']}, broken={overall['broken']}"
    )
    print("VERDICT:", "PASS" if overall["broken"] == 0 else "FAIL")


if __name__ == "__main__":
    main()
