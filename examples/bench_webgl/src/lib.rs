//! Deterministic Arabella performance benchmark with true GPU timing.
//!
//! Renders the three thesis test assets (Ghostscript Tiger, SVG Logo,
//! Bismillah) at a FIXED canvas resolution of 1080x520 — the same
//! resolution as the automated test harness in `tests/test.rs` — so the
//! numbers are reproducible across machines (modulo the GPU/CPU under test).
//!
//! Timing methodology:
//!   * CPU pre-processing (`Scene::reset` + every `fill`/`stroke`) is timed
//!     with `performance.now()`. These values sit well above the timer's
//!     ~0.1 ms quantum, so they are reliable.
//!   * GPU rasterization (`WebGlRenderer::render`) is timed with the
//!     `EXT_disjoint_timer_query_webgl2` extension, which reports the actual
//!     elapsed time on the GPU timeline in NANOSECONDS — not the CPU-side
//!     submission cost. Query results are read back asynchronously a few
//!     frames after submission, so the whole benchmark is driven by
//!     `requestAnimationFrame` as a state machine. Samples taken during a
//!     GPU disjoint event (context interruption) are discarded.
//!   * If the extension is unavailable, the harness falls back to timing
//!     `render()` with `performance.now()` (CPU-side submission cost) and
//!     flags this clearly in the report.
//!
//! Run from the workspace root:
//!   cargo run_wasm -p bench_webgl --release
//!
//! Read the Markdown table printed to the browser console (and on the
//! page). Copy the values into Tabel 4.4 of the thesis, and RECORD the
//! machine spec (CPU, GPU, OS, Chrome version) alongside them — the numbers
//! are only meaningful relative to a documented configuration.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "truncation/precision loss is acceptable in a benchmark harness"
)]

use core::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use lyon_geom::euclid::{Transform2D, UnknownUnit};
use lyon_path::{FillRule, Path, geom::point};
use peniko::Color;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlQuery};

use arabella::{Item, PicoSvg, RenderSize, Scene, WebGlRenderer};

// ── Benchmark configuration ──────────────────────────────────────────────
const BENCH_W: u16 = 1080;
const BENCH_H: u16 = 520;
/// Untimed frames rendered before measurement, to let shader compilation,
/// buffer allocation, and GPU pipeline state settle.
const WARMUP: u32 = 30;
/// Timed frames collected per asset.
const SAMPLES: u32 = 120;
/// Paint-op count above which an asset is treated as "heavy" and gets a
/// reduced warm-up / sample budget so the benchmark finishes in reasonable
/// time. The CPU work is deterministic, so a smaller sample still yields a
/// stable mean.
const HEAVY_OPS_THRESHOLD: usize = 5_000;
const HEAVY_WARMUP: u32 = 5;
const HEAVY_SAMPLES: u32 = 20;

/// Warm-up / sample budget for an asset given its paint-op count.
fn budget_for(op_count: usize) -> (u32, u32) {
    if op_count >= HEAVY_OPS_THRESHOLD {
        (HEAVY_WARMUP, HEAVY_SAMPLES)
    } else {
        (WARMUP, SAMPLES)
    }
}

// GL enums not exposed as named web-sys constants.
const TIME_ELAPSED_EXT: u32 = 0x88BF;
const GPU_DISJOINT_EXT: u32 = 0x8FBB;

type Tf = Transform2D<f32, UnknownUnit, UnknownUnit>;

// ── Paint-op collection (mirrors tests/test.rs document-order walk) ───────

enum PaintOp {
    Fill(FillItem),
    Stroke(StrokeFillItem),
}

struct FillItem {
    color: Color,
    path: Path,
    transform: Tf,
}

struct StrokeFillItem {
    color: Color,
    path: Path,
    style: kurbo::Stroke,
    transform: Tf,
}

fn collect_paint_ops(item: &Item, parent_transform: Tf, out: &mut Vec<PaintOp>) {
    match item {
        Item::Fill(fill) => {
            if fill.color.components[3] < 0.005 {
                return;
            }
            out.push(PaintOp::Fill(FillItem {
                color: fill.color,
                path: kurbo_to_lyon(&fill.path),
                transform: parent_transform,
            }));
        }
        Item::Stroke(stroke) => {
            if stroke.color.components[3] < 0.005 {
                return;
            }
            out.push(PaintOp::Stroke(StrokeFillItem {
                color: stroke.color,
                path: kurbo_to_lyon(&stroke.path),
                style: kurbo::Stroke::new(stroke.width),
                transform: parent_transform,
            }));
        }
        Item::Group(group) => {
            let c = group.affine.as_coeffs();
            let group_tf = Transform2D::new(
                c[0] as f32, c[1] as f32, c[2] as f32, c[3] as f32, c[4] as f32, c[5] as f32,
            );
            let combined = parent_transform.then(&group_tf);
            for child in &group.children {
                collect_paint_ops(child, combined, out);
            }
        }
    }
}

fn kurbo_to_lyon(bez: &kurbo::BezPath) -> Path {
    let mut builder = Path::builder();
    let mut open = false;
    for el in bez.elements() {
        match el {
            kurbo::PathEl::MoveTo(p) => {
                if open {
                    builder.end(false);
                }
                builder.begin(point(p.x as f32, p.y as f32));
                open = true;
            }
            kurbo::PathEl::LineTo(p) => {
                builder.line_to(point(p.x as f32, p.y as f32));
            }
            kurbo::PathEl::QuadTo(ctrl, to) => {
                builder.quadratic_bezier_to(
                    point(ctrl.x as f32, ctrl.y as f32),
                    point(to.x as f32, to.y as f32),
                );
            }
            kurbo::PathEl::CurveTo(c1, c2, to) => {
                builder.cubic_bezier_to(
                    point(c1.x as f32, c1.y as f32),
                    point(c2.x as f32, c2.y as f32),
                    point(to.x as f32, to.y as f32),
                );
            }
            kurbo::PathEl::ClosePath => {
                if open {
                    builder.end(true);
                    open = false;
                }
            }
        }
    }
    if open {
        builder.end(false);
    }
    builder.build()
}

fn build_scene(scene: &mut Scene, asset: &AssetData) {
    scene.reset();
    for op in &asset.ops {
        match op {
            PaintOp::Fill(f) => {
                let tf = f.transform.then(&asset.fit);
                scene.fill(&f.path, FillRule::NonZero, tf, f.color);
            }
            PaintOp::Stroke(s) => {
                let tf = s.transform.then(&asset.fit);
                scene.stroke(&s.path, &s.style, tf, s.color);
            }
        }
    }
}

// ── Statistics ────────────────────────────────────────────────────────────

struct Stats {
    min: f64,
    median: f64,
    mean: f64,
    n: usize,
}

fn summarize(samples: &[f64]) -> Stats {
    if samples.is_empty() {
        return Stats { min: 0.0, median: 0.0, mean: 0.0, n: 0 };
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    let min = sorted[0];
    let median = if n % 2 == 0 {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    };
    let mean = sorted.iter().sum::<f64>() / n as f64;
    Stats { min, median, mean, n }
}

struct AssetResult {
    name: &'static str,
    paint_ops: usize,
    tiles: usize,
    cpu: Stats,
    gpu: Stats,
    gpu_disjoint_discarded: usize,
}

struct AssetData {
    name: &'static str,
    ops: Vec<PaintOp>,
    /// Per-asset fit transform: scales+centers the geometry into the canvas
    /// with a margin and applies the Y-flip, computed from the geometry's
    /// own bounding box so no asset is clipped regardless of its viewBox.
    fit: Tf,
}

/// Union bounding box of all paint-op geometry, in post-group/pre-fit space
/// (i.e. each path point transformed by its own op transform). Returns
/// (min_x, min_y, max_x, max_y), or None if there are no points.
fn paint_ops_bbox(ops: &[PaintOp]) -> Option<(f32, f32, f32, f32)> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    let mut acc = |tf: &Tf, p: lyon_path::geom::Point<f32>| {
        let q = tf.transform_point(p);
        min_x = min_x.min(q.x);
        min_y = min_y.min(q.y);
        max_x = max_x.max(q.x);
        max_y = max_y.max(q.y);
    };

    for op in ops {
        let (path, tf) = match op {
            PaintOp::Fill(f) => (&f.path, &f.transform),
            PaintOp::Stroke(s) => (&s.path, &s.transform),
        };
        for ev in path.iter() {
            use lyon_path::PathEvent;
            match ev {
                PathEvent::Begin { at } => acc(tf, at),
                PathEvent::Line { from, to } => {
                    acc(tf, from);
                    acc(tf, to);
                }
                PathEvent::Quadratic { from, ctrl, to } => {
                    acc(tf, from);
                    acc(tf, ctrl);
                    acc(tf, to);
                }
                PathEvent::Cubic { from, ctrl1, ctrl2, to } => {
                    acc(tf, from);
                    acc(tf, ctrl1);
                    acc(tf, ctrl2);
                    acc(tf, to);
                }
                PathEvent::End { last, first, .. } => {
                    acc(tf, last);
                    acc(tf, first);
                }
            }
        }
    }

    if min_x.is_finite() && max_x > min_x && max_y > min_y {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

/// Build a transform that maps a geometry bounding box into the benchmark
/// canvas with a uniform margin, preserving aspect ratio and flipping Y so
/// the SVG y-down coordinate space lands right-side-up.
fn fit_transform(bbox: (f32, f32, f32, f32)) -> Tf {
    const MARGIN: f32 = 16.0;
    let (min_x, min_y, max_x, max_y) = bbox;
    let bw = (max_x - min_x).max(1e-6);
    let bh = (max_y - min_y).max(1e-6);
    let avail_w = (BENCH_W as f32 - 2.0 * MARGIN).max(1.0);
    let avail_h = (BENCH_H as f32 - 2.0 * MARGIN).max(1.0);
    let s = (avail_w / bw).min(avail_h / bh);
    // Center within the canvas.
    let content_w = bw * s;
    let content_h = bh * s;
    let off_x = (BENCH_W as f32 - content_w) * 0.5;
    let off_y = (BENCH_H as f32 - content_h) * 0.5;
    // x' = s*x + (off_x - s*min_x)
    // y' = -s*y + (BENCH_H - off_y + s*min_y)   (Y-flip)
    Tf::new(
        s,
        0.0,
        0.0,
        -s,
        off_x - s * min_x,
        BENCH_H as f32 - off_y + s * min_y,
    )
}

// ── State machine phases ─────────────────────────────────────────────────

enum Phase {
    Warmup(u32),
    Sample,
    Done,
}

struct BenchState {
    gl: WebGl2RenderingContext,
    timer_ext: bool,
    scene: Scene,
    renderer: WebGlRenderer,
    perf: web_sys::Performance,
    render_size: RenderSize,

    assets: Vec<AssetData>,
    /// Assets captured as PNG only (not timed); used for Bab 4.6 illustrations.
    capture_only: Vec<AssetData>,
    asset_idx: usize,
    phase: Phase,

    issued: u32,
    /// Number of timed samples to collect for the current asset (set when a
    /// warm-up phase begins, based on the asset's paint-op count).
    cur_samples: u32,
    pending: VecDeque<(WebGlQuery, f64)>, // (gpu query, cpu_ms recorded at submit)
    cpu_samples: Vec<f64>,
    gpu_samples: Vec<f64>,
    disjoint_discarded: usize,
    current_tiles: usize,

    results: Vec<AssetResult>,
    document: web_sys::Document,
    body: web_sys::HtmlElement,
    user_agent: String,
    canvas: HtmlCanvasElement,
}

impl BenchState {
    fn reset_asset_counters(&mut self) {
        self.issued = 0;
        self.pending.clear();
        self.cpu_samples.clear();
        self.gpu_samples.clear();
        self.disjoint_discarded = 0;
    }

    /// Build the scene + render, timing CPU with performance.now and GPU
    /// with a timer query (queued for async readback). Returns nothing; the
    /// GPU sample is collected later in `drain_ready`.
    fn submit_sample(&mut self) {
        let cpu_start = self.perf.now();
        build_scene(&mut self.scene, &self.assets[self.asset_idx]);
        let cpu_end = self.perf.now();
        let cpu_ms = cpu_end - cpu_start;

        if self.timer_ext {
            let query = self.gl.create_query().expect("create_query");
            self.gl.begin_query(TIME_ELAPSED_EXT, &query);
            self.renderer.render(&self.scene, &self.render_size);
            self.gl.end_query(TIME_ELAPSED_EXT);
            self.pending.push_back((query, cpu_ms));
        } else {
            // Fallback: CPU-side submission cost.
            let gpu_start = self.perf.now();
            self.renderer.render(&self.scene, &self.render_size);
            let gpu_end = self.perf.now();
            self.cpu_samples.push(cpu_ms);
            self.gpu_samples.push(gpu_end - gpu_start);
        }
        self.issued += 1;
    }

    /// Read back any GPU timer queries whose results are ready (FIFO).
    fn drain_ready(&mut self) {
        if !self.timer_ext {
            return;
        }
        let disjoint = self
            .gl
            .get_parameter(GPU_DISJOINT_EXT)
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        loop {
            let Some((query, _)) = self.pending.front() else {
                break;
            };
            let available = self
                .gl
                .get_query_parameter(query, WebGl2RenderingContext::QUERY_RESULT_AVAILABLE)
                .as_bool()
                .unwrap_or(false);
            if !available {
                break;
            }
            let (query, cpu_ms) = self.pending.pop_front().unwrap();
            let ns = self
                .gl
                .get_query_parameter(&query, WebGl2RenderingContext::QUERY_RESULT)
                .as_f64()
                .unwrap_or(0.0);
            self.gl.delete_query(Some(&query));

            // CPU sample always kept; GPU sample dropped if disjoint fired.
            self.cpu_samples.push(cpu_ms);
            if disjoint {
                self.disjoint_discarded += 1;
            } else {
                self.gpu_samples.push(ns / 1.0e6); // ns → ms
            }
        }
    }

    fn finalize_asset(&mut self) {
        let asset = &self.assets[self.asset_idx];
        let r = AssetResult {
            name: asset.name,
            paint_ops: asset.ops.len(),
            tiles: self.current_tiles,
            cpu: summarize(&self.cpu_samples),
            gpu: summarize(&self.gpu_samples),
            gpu_disjoint_discarded: self.disjoint_discarded,
        };
        web_sys::console::log_1(
            &format!(
                "[bench]   {} — ops={}, tiles={}, CPU mean={:.4} ms, GPU mean={:.4} ms (n_gpu={}, disjoint_dropped={})",
                r.name, r.paint_ops, r.tiles, r.cpu.mean, r.gpu.mean, r.gpu.n, r.gpu_disjoint_discarded
            )
            .into(),
        );
        self.results.push(r);
    }

    /// One animation-frame tick. Returns true when the whole benchmark is done.
    fn tick(&mut self) -> bool {
        match self.phase {
            Phase::Warmup(left) => {
                build_scene(&mut self.scene, &self.assets[self.asset_idx]);
                self.renderer.render(&self.scene, &self.render_size);
                if left <= 1 {
                    // Capture tile count for the upcoming sampling run.
                    build_scene(&mut self.scene, &self.assets[self.asset_idx]);
                    self.current_tiles = self.scene.tiles().len();
                    self.reset_asset_counters();
                    let (_, samples) = budget_for(self.assets[self.asset_idx].ops.len());
                    self.cur_samples = samples;
                    web_sys::console::log_1(
                        &format!(
                            "[bench] measuring {} … ({} samples)",
                            self.assets[self.asset_idx].name, samples
                        )
                        .into(),
                    );
                    self.phase = Phase::Sample;
                } else {
                    self.phase = Phase::Warmup(left - 1);
                }
                false
            }
            Phase::Sample => {
                self.drain_ready();
                if self.issued < self.cur_samples {
                    self.submit_sample();
                }
                let done_issuing = self.issued >= self.cur_samples;
                let drained = self.pending.is_empty();
                if done_issuing && drained {
                    self.finalize_asset();
                    self.asset_idx += 1;
                    if self.asset_idx < self.assets.len() {
                        let (warmup, _) =
                            budget_for(self.assets[self.asset_idx].ops.len());
                        self.phase = Phase::Warmup(warmup);
                    } else {
                        self.phase = Phase::Done;
                    }
                }
                false
            }
            Phase::Done => {
                self.report();
                self.capture_images();
                true
            }
        }
    }

    fn report(&self) {
        let method = if self.timer_ext {
            "GPU timer query (EXT_disjoint_timer_query_webgl2, true GPU-timeline nanoseconds)"
        } else {
            "performance.now() around render() (CPU-side submission cost — extension unavailable)"
        };
        let mut s = String::new();
        s.push_str(&format!(
            "Arabella benchmark — fixed {BENCH_W}x{BENCH_H}; light assets: warm-up {WARMUP} + {SAMPLES} samples, heavy assets (>{HEAVY_OPS_THRESHOLD} ops): warm-up {HEAVY_WARMUP} + {HEAVY_SAMPLES} samples\n"
        ));
        s.push_str(&format!("GPU timing method: {method}\n"));
        s.push_str("(times in ms; FPS derived from mean CPU+GPU total frame time)\n\n");
        s.push_str(
            "| Aset | Paint Ops | Tiles | CPU ms (min/med/mean) | GPU ms (min/med/mean) | Total ms (mean) | FPS |\n",
        );
        s.push_str(
            "|------|----------:|------:|-----------------------|-----------------------|----------------:|----:|\n",
        );
        for r in &self.results {
            let total = r.cpu.mean + r.gpu.mean;
            let fps = if total > 0.0 { 1000.0 / total } else { 0.0 };
            s.push_str(&format!(
                "| {} | {} | {} | {:.4} / {:.4} / {:.4} | {:.4} / {:.4} / {:.4} | {:.4} | {:.1} |\n",
                r.name,
                r.paint_ops,
                r.tiles,
                r.cpu.min,
                r.cpu.median,
                r.cpu.mean,
                r.gpu.min,
                r.gpu.median,
                r.gpu.mean,
                total,
                fps,
            ));
        }

        web_sys::console::log_1(&format!("\n{s}").into());
        web_sys::console::log_1(&"[bench] done.".into());

        if let Ok(pre) = self.document.create_element("pre") {
            let pre: web_sys::HtmlElement = pre.dyn_into().unwrap();
            pre.set_inner_text(&format!("{s}\nUA: {}\n", self.user_agent));
            pre.style().set_property("font-size", "13px").ok();
            pre.style().set_property("padding", "12px").ok();
            self.body.append_child(&pre).ok();
        }
    }

    /// Render each asset once more and capture the canvas as a PNG data URL,
    /// shown on-page as a downloadable thumbnail. Used to produce the visual
    /// figures for Subbab 4.3 (benchmarked assets) and Subbab 4.6
    /// (capture-only assets that illustrate parser limitations).
    ///
    /// The WebGL context is created without `preserveDrawingBuffer`, so the
    /// drawing buffer is cleared once the browser composites the frame. We
    /// therefore render and call `to_data_url` synchronously within this same
    /// tick, before yielding back to the compositor, so the buffer is still
    /// intact when read.
    fn capture_images(&mut self) {
        // Verification figures (Subbab 4.3): benchmarked assets.
        self.add_heading(
            "Output rendering Arabella — verifikasi visual 4.3 (klik tiap gambar untuk mengunduh PNG):",
        );
        let n = self.assets.len();
        for i in 0..n {
            // Build + render this asset, then immediately snapshot.
            build_scene(&mut self.scene, &self.assets[i]);
            self.renderer.render(&self.scene, &self.render_size);
            let name = self.assets[i].name;
            self.snapshot_canvas(name);
        }

        // Limitation figures (Subbab 4.6): capture-only assets.
        if !self.capture_only.is_empty() {
            self.add_heading(
                "Ilustrasi keterbatasan parser — Bab 4.6 (klik tiap gambar untuk mengunduh PNG):",
            );
            let m = self.capture_only.len();
            for i in 0..m {
                build_scene(&mut self.scene, &self.capture_only[i]);
                self.renderer.render(&self.scene, &self.render_size);
                let name = self.capture_only[i].name;
                self.snapshot_canvas(name);
            }
        }

        web_sys::console::log_1(&"[bench] image capture done.".into());
    }

    /// Append a section heading to the page.
    fn add_heading(&self, text: &str) {
        if let Ok(heading) = self.document.create_element("h3") {
            let heading: web_sys::HtmlElement = heading.dyn_into().unwrap();
            heading.set_inner_text(text);
            self.body.append_child(&heading).ok();
        }
    }

    /// Read the current canvas as a PNG data URL and append it to the page as
    /// a downloadable thumbnail. Must be called right after `render()` in the
    /// same tick (no `preserveDrawingBuffer`).
    fn snapshot_canvas(&self, name: &str) {
        let data_url = match self.canvas.to_data_url_with_type("image/png") {
            Ok(u) => u,
            Err(_) => {
                web_sys::console::warn_1(
                    &format!("[bench] toDataURL failed for {name}").into(),
                );
                return;
            }
        };

        // Wrap the image in an <a download> so a click saves the PNG.
        if let (Ok(anchor), Ok(img)) = (
            self.document.create_element("a"),
            self.document.create_element("img"),
        ) {
            let anchor: web_sys::HtmlElement = anchor.dyn_into().unwrap();
            anchor.set_attribute("href", &data_url).ok();
            let file_name = format!("arabella_{}.png", name.replace(".svg", ""));
            anchor.set_attribute("download", &file_name).ok();
            anchor.set_attribute("title", &format!("Unduh {file_name}")).ok();
            anchor.style().set_property("display", "inline-block").ok();
            anchor.style().set_property("margin", "8px").ok();

            let img: web_sys::HtmlElement = img.dyn_into().unwrap();
            img.set_attribute("src", &data_url).ok();
            img.set_attribute("alt", name).ok();
            img.style().set_property("border", "1px solid #444").ok();
            img.style().set_property("display", "block").ok();
            img.style().set_property("max-width", "540px").ok();

            anchor.append_child(&img).ok();
            self.body.append_child(&anchor).ok();
        }

        web_sys::console::log_1(
            &format!("[bench] captured PNG for {name} ({} bytes data URL)", data_url.len())
                .into(),
        );
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = requestAnimationFrame)]
    fn request_animation_frame(f: &Closure<dyn FnMut()>);
}

fn create_canvas(width: u32, height: u32) -> HtmlCanvasElement {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas: HtmlCanvasElement = document
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();
    canvas.set_width(width);
    canvas.set_height(height);
    canvas
        .style()
        .set_property("width", &format!("{width}px"))
        .unwrap();
    canvas
        .style()
        .set_property("height", &format!("{height}px"))
        .unwrap();
    canvas.style().set_property("border", "1px solid #444").ok();
    document.body().unwrap().append_child(&canvas).unwrap();
    canvas
}

/// Remove an XML DOCTYPE / DTD declaration from an SVG source string.
///
/// `PicoSvg::load` parses with roxmltree's default options, which reject any
/// document containing a DTD (`DtdDetected`). Some assets (e.g. el_gato.svg,
/// exported by older tools) carry a `<!DOCTYPE svg PUBLIC ...>` line. We strip
/// it here in the benchmark harness so the asset can be parsed; this does not
/// touch the core library and does not affect geometry.
fn strip_doctype(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<!DOCTYPE") {
        out.push_str(&rest[..start]);
        // DOCTYPE may contain an internal subset in [ ... ]; skip to the
        // matching '>' that closes the declaration (after any ']').
        let after = &rest[start..];
        let end = if let Some(br) = after.find('[') {
            // internal subset present: find ']' then the next '>'
            after[br..]
                .find(']')
                .and_then(|rb| after[br + rb..].find('>').map(|g| br + rb + g))
        } else {
            after.find('>')
        };
        match end {
            Some(e) => rest = &after[e + 1..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

fn load_asset(name: &'static str, svg: &str) -> AssetData {
    let cleaned = strip_doctype(svg);
    let pico = PicoSvg::load(&cleaned, 1.0).expect("Failed to parse SVG");
    // Collect with identity base so the stored op transforms are pure
    // geometry-space; the per-asset fit transform is composed at build time.
    let mut ops = Vec::new();
    for item in &pico.items {
        collect_paint_ops(item, Tf::identity(), &mut ops);
    }
    // Auto-fit so no asset is clipped regardless of its viewBox / size.
    let fit = match paint_ops_bbox(&ops) {
        Some(bbox) => fit_transform(bbox),
        None => Tf::identity(),
    };
    AssetData { name, ops, fit }
}

/// Run the full benchmark suite (async, rAF-driven) and print results.
pub fn run_benchmark() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    body.style().set_property("background-color", "#111").unwrap();
    body.style().set_property("color", "#eee").unwrap();
    body.style()
        .set_property("font-family", "ui-monospace, Menlo, Consolas, monospace")
        .unwrap();

    let perf = window.performance().unwrap();
    let canvas = create_canvas(BENCH_W as u32, BENCH_H as u32);
    let renderer = WebGlRenderer::new(&canvas);

    // Grab the SAME WebGL2 context the renderer created (get_context returns
    // the existing context for this canvas).
    let gl = canvas
        .get_context("webgl2")
        .expect("webgl2 context")
        .expect("webgl2 context present")
        .dyn_into::<WebGl2RenderingContext>()
        .expect("WebGl2RenderingContext");

    // Enable the GPU timer extension if available.
    let timer_ext = matches!(
        gl.get_extension("EXT_disjoint_timer_query_webgl2"),
        Ok(Some(_))
    );

    let dpr = window.device_pixel_ratio();
    web_sys::console::log_1(
        &format!(
            "[bench] starting — canvas {BENCH_W}x{BENCH_H} device px, devicePixelRatio={dpr:.2}, gpu_timer={timer_ext}"
        )
        .into(),
    );
    if !timer_ext {
        web_sys::console::warn_1(
            &"[bench] EXT_disjoint_timer_query_webgl2 NOT available; GPU times fall \
              back to CPU-side submission cost. Report will state this."
                .into(),
        );
    }

    // Assets that are BENCHMARKED (timed) and captured as PNG.
    let assets = vec![
        load_asset(
            "Ghostscript_Tiger.svg",
            include_str!("../../../assets/Ghostscript_Tiger.svg"),
        ),
        load_asset("el_gato.svg", include_str!("../../../assets/el_gato.svg")),
        load_asset("paris-30k.svg", include_str!("../../../assets/paris-30k.svg")),
    ];

    // Assets that are ONLY captured as PNG (not timed) — used to illustrate
    // parser limitations in Bab 4.6. These rely on unsupported SVG features
    // (SVG_Logo uses defs/use; bismillah uses pattern), so their output
    // deliberately differs from the browser reference.
    let capture_only = vec![
        load_asset("SVG_Logo.svg", include_str!("../../../assets/SVG_Logo.svg")),
        load_asset("bismillah.svg", include_str!("../../../assets/bismillah.svg")),
    ];

    let first_warmup = budget_for(assets[0].ops.len()).0;
    let state = Rc::new(RefCell::new(BenchState {
        gl,
        timer_ext,
        scene: Scene::new(BENCH_W, BENCH_H),
        renderer,
        perf,
        render_size: RenderSize {
            width: BENCH_W as u32,
            height: BENCH_H as u32,
        },
        assets,
        capture_only,
        asset_idx: 0,
        phase: Phase::Warmup(first_warmup),
        issued: 0,
        cur_samples: SAMPLES,
        pending: VecDeque::new(),
        cpu_samples: Vec::with_capacity(SAMPLES as usize),
        gpu_samples: Vec::with_capacity(SAMPLES as usize),
        disjoint_discarded: 0,
        current_tiles: 0,
        results: Vec::new(),
        document,
        body,
        user_agent: window.navigator().user_agent().unwrap_or_default(),
        canvas,
    }));

    // Self-referential rAF loop driving the state machine until Done.
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let state2 = state.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let done = state2.borrow_mut().tick();
        if !done {
            request_animation_frame(f.borrow().as_ref().unwrap());
        }
    }) as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());
    Box::leak(Box::new(g));
}
