#!/usr/bin/env python3
"""Spot-check critical references with actual line content."""
import os

ROOT = r"e:\Users\Documents\A - TUGAS BINUS\Thesis\Code"

CHECKS = [
    ("Cargo.toml", 7),
    ("Cargo.toml", 26),
    ("Cargo.toml", 48),
    ("Cargo.toml", 49),
    ("Cargo.toml", 50),
    ("Cargo.toml", 94),
    ("src/blocks.rs", 6),
    ("src/blocks.rs", 7),
    ("src/blocks.rs", 10),
    ("src/blocks.rs", 11),
    ("src/blocks.rs", 93),
    ("src/blocks.rs", 107),
    ("src/blocks.rs", 710),
    ("src/builder.rs", 17),
    ("src/builder.rs", 28),
    ("src/builder.rs", 29),
    ("src/builder.rs", 84),
    ("src/builder.rs", 151),
    ("src/render/shaders/render_tile.frag", 24),
    ("src/render/shaders/render_tile.frag", 90),
    ("src/render/shaders/render_tile.frag", 215),
    ("src/render/shaders/render_tile.frag", 218),
    ("src/render/shaders/render_tile.frag", 219),
    ("src/render/webgl.rs", 271),
    ("src/render/webgl.rs", 296),
    ("src/render/webgl.rs", 525),
    ("src/render/webgl.rs", 529),
    ("src/scene.rs", 70),
    ("src/scene.rs", 117),
    ("src/scene.rs", 158),
    ("src/scene.rs", 165),
    ("src/tile.rs", 9),
    ("src/tile.rs", 23),
    ("src/pico_svg.rs", 84),
    ("src/pico_svg.rs", 191),
    ("tests/test.rs", 147),
    ("tests/test.rs", 151),
    ("tests/test.rs", 152),
    ("examples/native_webgl/src/main.rs", 22),
    ("examples/native_webgl/src/main.rs", 27),
    ("examples/native_webgl/src/lib.rs", 316),
    ("examples/native_webgl/src/lib.rs", 325),
    ("examples/native_webgl/src/lib.rs", 335),
    ("examples/native_webgl/src/lib.rs", 336),
    ("examples/native_webgl/src/lib.rs", 337),
    ("examples/native_webgl/src/lib.rs", 338),
    ("examples/native_webgl/src/lib.rs", 341),
    ("src/path.rs", 7),
    ("src/path.rs", 352),
    ("src/path.rs", 391),
    ("src/flatten.rs", 18),
    ("src/flatten.rs", 20),
    ("src/flatten.rs", 31),
    ("src/flatten.rs", 62),
]

for path, line in CHECKS:
    abs_path = os.path.join(ROOT, path.replace("/", os.sep))
    with open(abs_path, encoding="utf-8") as fh:
        lines = fh.readlines()
    if line <= len(lines):
        content = lines[line - 1].rstrip()
        print(f"{path}:{line:>4}  >  {content}")
    else:
        print(f"{path}:{line:>4}  >  OUT OF RANGE (file has {len(lines)})")
