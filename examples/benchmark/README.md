# Arabella Benchmark Suite

Side-by-side performance comparison of arabella against two other vector
rendering approaches, all rendering the **same** Ghostscript Tiger SVG with
the **same** overlay (FPS / CPU ms / zoom) and the **same** pan/zoom/reset
controls.

## What's being compared

| Demo | Engine | Method | Backend |
|------|--------|--------|---------|
| `../native_webgl` | **arabella** | analytic tile coverage (hybrid CPU+GPU) | WebGL2 |
| `canvaskit/` | **PixiJS** (`preference: 'canvas'`) | browser Canvas2D path rasterizer | **Chrome's Skia** |
| `tessellation/` | **PixiJS** (`preference: 'webgl'`) | **earcut** ear-clipping tessellation → triangle mesh | WebGL |
| (separate) | **vello_hybrid** | sparse strips (hybrid CPU+GPU) | WebGL2 |

## Running

These are plain static HTML files — no build step. Serve the **arabella
repo root** (so the `examples/benchmark/shared/` assets resolve) with any
static server and open the pages:

```bash
# from the arabella/ workspace root
python -m http.server 8000
```

Then visit:

- Skia (Canvas2D):     http://localhost:8000/examples/benchmark/canvaskit/
- Tessellation (WebGL): http://localhost:8000/examples/benchmark/tessellation/

PixiJS 8.18.1 is loaded from the jsDelivr CDN, so you need an internet
connection the first time (the browser will cache it afterward).

For the **arabella** and **vello_hybrid** numbers, run their respective
`cargo run_wasm -p native_webgl --release` demos.

## Measurement methodology (read before trusting numbers)

For a *fair* comparison every engine must do the **same work per frame**.

- **arabella** re-bins (flatten + DDA + tile emit) every frame.
- **vello_hybrid** re-encodes (flatten + strip gen) every frame.
- **PixiJS** by default *caches* its tessellation/rasterization and only
  re-submits on transform change. That would be an unfair advantage (it
  would measure transform-only submit, not a full rebuild).

So the PixiJS harness **forces a per-frame rebuild** by re-assigning the
graphics context each frame (bumping its dirty id). This makes the overlay
number directly comparable to arabella's per-frame cost.

**The harness also logs BOTH numbers to the browser console** so the thesis
can present each engine honestly:

```
[bench] avg render over 60 frames — uncached(rebuild): X ms, cached(reuse tess): Y ms
```

- `uncached(rebuild)` ↔ comparable to arabella/vello (full rebuild each frame)
- `cached(reuse tess)` ↔ PixiJS's real-world best case (static scene, only camera moves)

### The "GPU ms" caveat

arabella and vello_hybrid split their overlay into CPU ms + GPU ms, where
"GPU ms" is really *CPU time spent submitting* WebGL work (WebGL doesn't
block on GPU completion). PixiJS doesn't expose a clean submit-only phase,
so its overlay shows a single combined **CPU+submit** number. When
comparing, add arabella's CPU + GPU columns to get the equivalent
combined figure.

## Important clarification on "CanvasKit / Skia"

PixiJS does **not** bundle CanvasKit (the standalone WASM Skia). The
`canvas` backend uses the browser's built-in `CanvasRenderingContext2D`.
In **Chrome**, that 2D context is implemented on top of **Skia** — the same
rasterizer CanvasKit wraps, just compiled into the browser instead of
shipped as WASM. So the `canvaskit/` demo is a legitimate Skia comparison
*on Chrome*. On Firefox it would route to Firefox's own 2D backend
(currently a mix of Skia / WebRender), so run it in **Chrome** for a true
Skia number.

PixiJS's Canvas renderer is also officially "experimental" — complex SVGs
may render with minor imperfections. It's fine for timing, but eyeball the
output before quoting visual-quality conclusions.
