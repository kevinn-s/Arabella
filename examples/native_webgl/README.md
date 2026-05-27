# Arabella Interactive Demo (WebGL2)

Interactive pan + zoom demo of arabella's hybrid CPU/GPU renderer running
in the browser via WebGL2. Mirrors the layout of `vello_hybrid`'s
`native_webgl` example so the build/run commands are familiar.

## Run

```
cargo run_wasm -p native_webgl --release
```

This builds a release-tuned wasm-bindgen cdylib (with LTO, `simd128`, no
test harness) and serves it locally. **Use this for performance
measurements.**

The `wasm-pack test` path adds 3–5× harness overhead and should only be
used for debugging.

## Controls

| Input            | Action                          |
|------------------|---------------------------------|
| Drag mouse       | Pan                             |
| Mouse wheel      | Zoom (centered on cursor)       |
| Space            | Reset view                      |
| ← / →            | Previous / next scene           |

## What's on screen

Top-left overlay shows:

- FPS (rolling 60-frame average) and frame interval in ms
- CPU time per frame: scene reset + flatten + DDA bin + tile emit
- GPU time per frame: WebGL submission + draw + readback wait
- Current camera zoom factor
- Number of paint ops in the current scene

## Optimization notes (release builds)

The workspace's `[profile.release]` enables:

- `lto = "fat"` for cross-crate inlining (kurbo / lyon / fearless_simd)
- `codegen-units = 1` so all functions see each other
- `panic = "abort"` to drop unwind tables from the hot path
- `strip = true` for a smaller wasm binary

Plus `+simd128` in `.cargo/config.toml` so fearless_simd's WASM SIMD
backend lights up.
