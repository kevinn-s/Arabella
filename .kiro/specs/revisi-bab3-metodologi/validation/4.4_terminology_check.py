"""Property 4 — Canonical Terminology Consistency check for Skripsi/bab3_metodologi.md.

Read-only validation. Splits the document into paragraphs (blank-line separated)
and reports per-paragraph occurrences of canonical and non-canonical terms.
"""
import re
import sys

SRC = r"e:\Users\Documents\A - TUGAS BINUS\Thesis\Code\Skripsi\bab3_metodologi.md"


def paragraphs_with_lines(text):
    paras = []
    cur_lines = []
    cur_start = 1
    for i, line in enumerate(text.split("\n"), start=1):
        if line.strip() == "":
            if cur_lines:
                paras.append((cur_start, i - 1, "\n".join(cur_lines)))
                cur_lines = []
            cur_start = i + 1
        else:
            if not cur_lines:
                cur_start = i
            cur_lines.append(line)
    if cur_lines:
        paras.append((cur_start, len(text.split("\n")), "\n".join(cur_lines)))
    return paras


def find_all(pattern, text, flags=re.IGNORECASE):
    return [m for m in re.finditer(pattern, text, flags)]


# Canonical and non-canonical patterns. Each entry: (label, pattern, kind)
# kind: "canonical" or "non_canonical"
PATTERNS = [
    # 1. binning DDA
    ("binning DDA", r"\bbinning\s+DDA\b", "canonical"),
    ("DDA binning (non-canonical)", r"\bDDA\s+binning\b", "non_canonical"),
    ("tile binning (non-canonical)", r"\btile[-\s]?binning\b", "non_canonical"),
    ("tiling binning (non-canonical)", r"\btiling\s+binning\b", "non_canonical"),
    # 2. akumulator signed-area
    ("akumulator signed-area", r"\bakumulator\s+signed-area\b", "canonical"),
    ("signed area accumulator (non-canonical)", r"\bsigned\s*area\s+accumulator\b", "non_canonical"),
    ("akumulator area bertanda (non-canonical)", r"\bakumulator\s+area\s+bertanda\b", "non_canonical"),
    # 3. propagasi backdrop
    ("propagasi backdrop", r"\bpropagasi\s+backdrop\b", "canonical"),
    ("backdrop propagation (non-canonical)", r"\bbackdrop\s+propagation\b", "non_canonical"),
    ("propagation backdrop (non-canonical)", r"\bpropagation\s+backdrop\b", "non_canonical"),
    ("propagasi winding number (non-canonical)", r"\bpropagasi\s+winding\s+number\b", "non_canonical"),
    # 4. fragment shader
    ("fragment shader (lowercase)", r"(?<!`)\bfragment\s+shader\b", "canonical"),
    ("Fragment Shader (title case)", r"\bFragment\s+Shader\b", "canonical_titlecase"),
    ("pixel shader (non-canonical)", r"\bpixel\s+shader\b", "non_canonical"),
    ("shader piksel (non-canonical)", r"\bshader\s+piksel\b", "non_canonical"),
    ("shader fragmen (non-canonical)", r"\bshader\s+fragmen\b", "non_canonical"),
    ("fragment program (non-canonical)", r"\bfragment\s+program\b", "non_canonical"),
    # 5. preprocessing vs pra-pemrosesan (paragraph-level mix forbidden)
    ("pra-pemrosesan", r"\bpra-?pemrosesan\b", "canonical_a"),  # note: tracks both spellings
    ("preprocessing", r"\bpreprocessing\b", "canonical_b"),
    ("pra-proses (non-canonical)", r"\bpra-?proses\b(?!an)", "non_canonical"),
    ("pra-pengolahan (non-canonical)", r"\bpra-?pengolahan\b", "non_canonical"),
    # 6. pipeline hibrida
    ("pipeline hibrida", r"\bpipeline\s+hibrida\b", "canonical"),
    ("hybrid pipeline (non-canonical)", r"\bhybrid\s+pipeline\b", "non_canonical"),
    ("pipeline hybrid (non-canonical)", r"\bpipeline\s+hybrid\b", "non_canonical"),
    ("pipa hibrida (non-canonical)", r"\bpipa\s+hibrida\b", "non_canonical"),
    # 7. rasterization pipeline tradisional / pipeline rasterisasi konvensional
    ("rasterization pipeline tradisional", r"\brasterization\s+pipeline\s+tradisional\b", "canonical"),
    ("pipeline rasterisasi konvensional", r"\bpipeline\s+rasterisasi\s+konvensional\b", "canonical"),
    ("pipeline rasterisasi tradisional (non-canonical mix)", r"\bpipeline\s+rasterisasi\s+tradisional\b", "non_canonical"),
    ("rasterization pipeline konvensional (non-canonical mix)", r"\brasterization\s+pipeline\s+konvensional\b", "non_canonical"),
    ("traditional rasterization pipeline (non-canonical)", r"\btraditional\s+rasterization\s+pipeline\b", "non_canonical"),
    ("conventional rasterization pipeline (non-canonical)", r"\bconventional\s+rasterization\s+pipeline\b", "non_canonical"),
    # 8. viewport
    ("viewport", r"\bviewport\b", "canonical"),
    ("kanvas tampilan (non-canonical synonym for viewport)", r"\bkanvas\s+tampilan\b", "non_canonical"),
    ("wilayah layar target (non-canonical synonym for viewport)", r"\bwilayah\s+layar\s+target\b", "non_canonical"),
    ("daerah layar target (non-canonical synonym for viewport)", r"\bdaerah\s+layar\s+target\b", "non_canonical"),
    ("area tampilan (non-canonical synonym for viewport)", r"\barea\s+tampilan\b", "non_canonical"),
    ("jendela tampilan (non-canonical synonym for viewport)", r"\bjendela\s+tampilan\b", "non_canonical"),
    # 9. winding number
    ("winding number", r"\bwinding\s+number\b", "canonical"),
    ("nomor winding (non-canonical)", r"\bnomor\s+winding\b", "non_canonical"),
    ("bilangan winding (non-canonical)", r"\bbilangan\s+winding\b", "non_canonical"),
    ("angka winding (non-canonical)", r"\bangka\s+winding\b", "non_canonical"),
    ("bilangan lintas (non-canonical)", r"\bbilangan\s+lintas\b", "non_canonical"),
    ("winding count (non-canonical)", r"\bwinding\s+count\b", "non_canonical"),
]


def main():
    with open(SRC, encoding="utf-8") as f:
        text = f.read()

    paras = paragraphs_with_lines(text)
    print(f"# 4.4 Canonical Terminology Property — Paragraph-Level Scan")
    print(f"Total paragraphs: {len(paras)}")
    print()

    non_canonical_hits = []
    pp_mix_hits = []  # paragraphs with both pra-pemrosesan and preprocessing
    canonical_summary = {}

    for label, pat, kind in PATTERNS:
        canonical_summary[label] = 0

    for p_idx, (start, end, body) in enumerate(paras, start=1):
        has_prapemrosesan = bool(re.search(r"\bpra-?pemrosesan\b", body, re.IGNORECASE))
        has_preprocessing = bool(re.search(r"\bpreprocessing\b", body, re.IGNORECASE))
        if has_prapemrosesan and has_preprocessing:
            pp_mix_hits.append((p_idx, start, end, body))

        for label, pat, kind in PATTERNS:
            matches = find_all(pat, body)
            if matches:
                canonical_summary[label] += len(matches)
                if kind == "non_canonical":
                    non_canonical_hits.append((label, p_idx, start, end, [m.group(0) for m in matches], body))

    print("## Canonical Term Counts (whole document)")
    for label, count in canonical_summary.items():
        print(f"  - {label}: {count}")
    print()

    print("## Paragraphs that mix `pra-pemrosesan` and `preprocessing` (Req 12.3 violation)")
    if pp_mix_hits:
        for p_idx, start, end, body in pp_mix_hits:
            print(f"  - Paragraph {p_idx} (lines {start}-{end})")
            print(f"    excerpt: {body[:200]}...")
    else:
        print("  NONE — Property satisfied.")
    print()

    print("## Non-canonical synonym hits")
    if non_canonical_hits:
        for label, p_idx, start, end, hits, body in non_canonical_hits:
            print(f"  - [{label}] paragraph {p_idx} (lines {start}-{end}): {hits}")
            print(f"    excerpt: {body[:200]}...")
    else:
        print("  NONE — Property satisfied.")
    print()


if __name__ == "__main__":
    main()
