#!/usr/bin/env python3
"""Spot-check critical line-range references with the start/end content."""
import os

ROOT = r"e:\Users\Documents\A - TUGAS BINUS\Thesis\Code"

CHECKS = [
    ("src/tile.rs", 9, 23),
    ("src/scene.rs", 35, 41),
    ("src/scene.rs", 155, 157),
    ("src/scene.rs", 165, 168),
    ("src/builder.rs", 36, 54),
    ("src/builder.rs", 60, 61),
    ("src/builder.rs", 138, 148),
    ("src/builder.rs", 158, 162),
    ("src/builder.rs", 193, 247),
    ("src/builder.rs", 223, 237),
    ("src/builder.rs", 240, 247),
    ("src/builder.rs", 360, 369),
    ("src/builder.rs", 196, 203),
    ("src/builder.rs", 43, 45),
    ("src/builder.rs", 40, 42),
    ("src/builder.rs", 151, 337),
    ("src/blocks.rs", 6, 7),
    ("src/blocks.rs", 10, 11),
    ("src/blocks.rs", 21, 39),
    ("src/blocks.rs", 51, 55),
    ("src/blocks.rs", 93, 102),
    ("src/blocks.rs", 107, 160),
    ("src/blocks.rs", 115, 117),
    ("src/blocks.rs", 134, 138),
    ("src/blocks.rs", 149, 153),
    ("src/blocks.rs", 163, 207),
    ("src/blocks.rs", 210, 254),
    ("src/blocks.rs", 257, 301),
    ("src/blocks.rs", 304, 348),
    ("src/blocks.rs", 353, 391),
    ("src/blocks.rs", 366, 374),
    ("src/blocks.rs", 394, 451),
    ("src/blocks.rs", 454, 505),
    ("src/blocks.rs", 508, 560),
    ("src/blocks.rs", 563, 619),
    ("src/blocks.rs", 625, 657),
    ("src/blocks.rs", 664, 669),
    ("src/blocks.rs", 710, 757),
    ("src/render/webgl.rs", 265, 269),
    ("src/render/webgl.rs", 381, 388),
    ("src/render/webgl.rs", 393, 398),
    ("src/render/webgl.rs", 423, 446),
    ("src/render/shaders/render_tile.frag", 15, 21),
    ("src/render/shaders/render_tile.frag", 213, 220),
    ("src/render/shaders/render_tile.frag", 218, 219),
    ("src/path.rs", 352, 375),
    ("src/path.rs", 391, 475),
    ("src/flatten.rs", 20, 29),
    ("src/flatten.rs", 31, 58),
    ("src/flatten.rs", 46, 57),
    ("src/flatten.rs", 62, 87),
    ("src/pico_svg.rs", 28, 34),
    ("src/pico_svg.rs", 36, 44),
    ("examples/native_webgl/src/main.rs", 22, 27),
    ("examples/native_webgl/src/lib.rs", 316, 341),
    ("Cargo.toml", 28, 41),
    ("Cargo.toml", 48, 50),
    ("Cargo.toml", 57, 87),
]

for path, a, b in CHECKS:
    abs_path = os.path.join(ROOT, path.replace("/", os.sep))
    with open(abs_path, encoding="utf-8") as fh:
        lines = fh.readlines()
    if b > len(lines):
        print(f"FAIL {path}:{a}-{b} -- file has {len(lines)}")
        continue
    start_line = lines[a - 1].rstrip()
    end_line = lines[b - 1].rstrip()
    print(f"{path}:{a}-{b}")
    print(f"   START L{a}: {start_line[:120]}")
    print(f"   END   L{b}: {end_line[:120]}")
