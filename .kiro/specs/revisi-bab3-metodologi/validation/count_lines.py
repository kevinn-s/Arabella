#!/usr/bin/env python3
"""Count number of lines in each relevant source file."""
import os

ROOT = r"e:\Users\Documents\A - TUGAS BINUS\Thesis\Code"
FILES = [
    "Cargo.toml",
    "src/pico_svg.rs",
    "src/path.rs",
    "src/flatten.rs",
    "src/blocks.rs",
    "src/builder.rs",
    "src/scene.rs",
    "src/tile.rs",
    "src/render/webgl.rs",
    "src/render/shaders/render_tile.frag",
    "src/render/shaders/render_tile.vert",
    "src/render/common.rs",
    "src/render/mod.rs",
    "src/lib.rs",
    "src/paint/mod.rs",
    "tests/test.rs",
    "examples/native_webgl/src/main.rs",
    "examples/native_webgl/src/lib.rs",
    ".cargo/config.toml",
]

for f in FILES:
    p = os.path.join(ROOT, f.replace("/", os.sep))
    if os.path.exists(p):
        with open(p, encoding="utf-8") as fh:
            n = sum(1 for _ in fh)
        print(f"{n:6d}  {f}")
    else:
        print(f"  MISSING  {f}")
