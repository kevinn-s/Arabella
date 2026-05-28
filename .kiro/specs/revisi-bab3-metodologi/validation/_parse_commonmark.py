"""Parse Skripsi/bab3_metodologi.md as CommonMark and report any errors.

The reference CommonMark spec does not define hard "errors" in a strict sense
(every byte sequence parses), but we approximate parsability checks by:

1. Loading the file as UTF-8 (catch decoding errors).
2. Running the commonmark parser end-to-end (catch any exceptions).
3. Rendering the resulting AST to HTML (catch any rendering exceptions).
4. Sanity-checking that all opened code fences are closed (unbalanced fences
   are a common authoring mistake CommonMark silently accepts but flags as
   informationally suspicious).

Outputs are written to stdout. Exit code 0 = PASS, 1 = FAIL.
"""

from __future__ import annotations

import re
import sys
import traceback
from pathlib import Path

import commonmark

TARGET = Path(
    r"e:\\Users\\Documents\\A - TUGAS BINUS\\Thesis\\Code\\Skripsi\\bab3_metodologi.md"
)


def main() -> int:
    if not TARGET.exists():
        print(f"FAIL: target file not found: {TARGET}")
        return 1

    raw = TARGET.read_bytes()
    print(f"file_size_bytes={len(raw)}")

    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as exc:
        print(f"FAIL: file is not valid UTF-8: {exc}")
        return 1
    print(f"utf8_decoded_chars={len(text)}")

    line_count = text.count("\n") + (0 if text.endswith("\n") else 1)
    print(f"line_count={line_count}")

    parser = commonmark.Parser()
    renderer = commonmark.HtmlRenderer()
    try:
        ast = parser.parse(text)
    except Exception:
        print("FAIL: commonmark parser raised an exception:")
        traceback.print_exc()
        return 1

    try:
        html = renderer.render(ast)
    except Exception:
        print("FAIL: commonmark renderer raised an exception:")
        traceback.print_exc()
        return 1

    print(f"html_render_chars={len(html)}")

    # Heading inventory
    walker = ast.walker()
    headings: list[tuple[int, str]] = []
    event = walker.nxt()
    while event is not None:
        node, entering = event["node"], event["entering"]
        if node.t == "heading" and entering:
            child = node.first_child
            chunks: list[str] = []
            while child is not None:
                if child.t == "text":
                    chunks.append(child.literal or "")
                child = child.nxt
            headings.append((node.level, "".join(chunks)))
        event = walker.nxt()

    print(f"heading_count={len(headings)}")
    print("first_5_headings:")
    for lvl, txt in headings[:5]:
        print(f"  H{lvl}: {txt}")

    # Fence balance sanity check
    fence_lines = [
        ln for ln in text.splitlines()
        if re.match(r"^\s{0,3}(```|~~~)", ln)
    ]
    fence_imbalance = len(fence_lines) % 2
    print(f"fence_line_count={len(fence_lines)}")
    print(f"fence_imbalance={fence_imbalance}  # 0 = balanced")

    # First-line check
    first_line = text.splitlines()[0] if text else ""
    expected_first = "# BAB 3 METODE PENELITIAN"
    first_line_ok = first_line == expected_first
    print(f"first_line_match={first_line_ok}")
    if not first_line_ok:
        print(f"  expected: {expected_first!r}")
        print(f"  actual:   {first_line!r}")

    if not first_line_ok:
        return 1
    if fence_imbalance:
        print("FAIL: unbalanced code fences detected")
        return 1

    print("PASS: file decodes as UTF-8, parses as CommonMark without exceptions, "
          "renders to HTML, fences balanced, first line correct.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
