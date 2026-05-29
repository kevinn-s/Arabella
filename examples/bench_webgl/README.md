# Arabella Performance Benchmark (WebGL2)

Deterministic, reproducible performance harness for arabella's hybrid
CPU/GPU renderer. Produces the numbers reported in **Tabel 4.4** of the
thesis (Bab 4). Unlike the interactive demo (`native_webgl`), this harness
fixes the canvas resolution and averages many frames so the results are
reproducible across runs and machines.

## What it measures

For each of the three test assets (Ghostscript Tiger, el gato, paris-30k),
at a fixed **1080×520** canvas:

- **CPU ms** — pre-processing time: `Scene::reset` + every `fill`/`stroke`
  (flatten + DDA bin + tile emit), timed with `performance.now()`.
- **GPU ms** — true GPU-timeline rasterization time, measured with the
  `EXT_disjoint_timer_query_webgl2` extension (nanosecond resolution), NOT
  the CPU-side submission cost. Samples taken during a GPU disjoint event
  are discarded.
- **Tiles** — number of nontrivial tiles emitted by pre-processing.
- **FPS** — derived from the reciprocal of mean total frame time (CPU+GPU).

Each asset runs warm-up frames (untimed) then timed frames; the report shows
min / median / mean for CPU and GPU. Light assets use 30 warm-up + 120
samples; heavy assets (>5000 paint ops, e.g. paris-30k) use a reduced 5 + 20
budget so the run finishes in reasonable time — the CPU work is deterministic
so a smaller sample still yields a stable mean. The geometry of each asset is
auto-fit (scaled + centered with a margin) into the 1080×520 canvas, so no
asset is clipped regardless of its viewBox.

If `EXT_disjoint_timer_query_webgl2` is unavailable, the harness falls back
to timing `render()` with `performance.now()` and **states this in the
report header** so the numbers are not misread.

## Run

From the workspace root:

```
cargo run_wasm -p bench_webgl --release
```

This builds a release-tuned wasm-bindgen cdylib (LTO, `codegen-units = 1`,
`+simd128`, no test harness) and serves it locally — the same release
profile as the prototype, so the numbers reflect a properly inlined build
rather than the 3–5× slower `wasm-pack test` harness.

Then open the served URL **in Google Chrome 113+** (copy the
`http://localhost:PORT` URL printed in the terminal if Chrome is not your
default browser). The benchmark runs automatically on page load. Results
appear in two places:

1. **DevTools Console** (F12 → Console) — a ready-to-copy Markdown table.
2. **On the page** (a `<pre>` block) — the same table plus the user-agent
   string.

Wait until the console prints `[bench] done.`

## Before you trust the numbers

1. **Confirm GPU acceleration.** Open `chrome://gpu` and check that
   **WebGL: Hardware accelerated**. If it shows software (SwiftShader), the
   GPU numbers are not representative — record that as a limitation.
2. **Confirm the timer extension is active.** The console line
   `[bench] starting — ... gpu_timer=true` must say `true`. If it says
   `false`, GPU times are CPU-side submission cost (the report header will
   also say so).
3. **Confirm clean samples.** Each asset line should show `disjoint_dropped=0`
   (or a small number). Large drops mean GPU scheduling interruptions.
4. **Stabilize the machine.** Close heavy apps, plug into AC power (disable
   battery-saver), and run **2–3 times**, taking a consistent run (or the
   mean across runs, as the thesis does).

## Record the machine spec

The numbers are only meaningful relative to a documented configuration.
Record the following alongside the results (this is what populates
**Tabel 4.1** in the thesis):

```
CPU            : <e.g. Intel Core i7-9700K @ 3.60 GHz, 8C/8T>
GPU            : <e.g. AMD Radeon RX 5700 XT>
RAM            : <e.g. 16 GB>
OS             : <e.g. Windows 11 Pro, build 26200>
Browser        : <e.g. Google Chrome 148.0.7778.179 (64-bit)>
WebGL accel    : <Hardware accelerated / software — from chrome://gpu>
gpu_timer      : <true / false — from the [bench] starting console line>
Date measured  : <YYYY-MM-DD>
```

On Windows, you can collect most of this from PowerShell:

```powershell
(Get-CimInstance Win32_Processor).Name
(Get-CimInstance Win32_VideoController).Name
[math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory/1GB,1)
(Get-CimInstance Win32_OperatingSystem).Caption
```

Chrome version: open `chrome://version` (top line).

## Configuration

Tunable constants live at the top of `src/lib.rs`:

| Constant   | Default | Meaning                              |
|------------|---------|--------------------------------------|
| `BENCH_W`  | 1080    | Canvas width (device pixels)         |
| `BENCH_H`  | 520     | Canvas height (device pixels)        |
| `WARMUP`   | 30      | Untimed warm-up frames (light asset) |
| `SAMPLES`  | 120     | Timed frames (light asset)           |
| `HEAVY_OPS_THRESHOLD` | 5000 | Paint-op count above which an asset is "heavy" |
| `HEAVY_WARMUP`  | 5  | Untimed warm-up frames (heavy asset) |
| `HEAVY_SAMPLES` | 20 | Timed frames (heavy asset)           |

The fixed 1080×520 matches `tests/test.rs` so the workload is identical to
the automated correctness test. `devicePixelRatio != 1` does not change the
workload — the canvas backing store is forced to 1080×520 device pixels
regardless.

## Notes

- This crate does not modify the core `arabella` library in `src/`; it only
  consumes its public API (`Scene`, `WebGlRenderer`, `RenderSize`).
- The `favicon.ico not found` message from cargo-run-wasm is harmless.
