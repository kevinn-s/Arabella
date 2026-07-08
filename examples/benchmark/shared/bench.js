// Shared benchmark harness for the PixiJS-based comparison demos.
//
// Both the "canvaskit" (browser Canvas2D → Chrome's Skia) and
// "tessellation" (PixiJS WebGL → earcut) demos import this module and call
// `runBenchmark(...)` with a renderer preference. The harness:
//
//   - loads PixiJS 8.x from CDN
//   - fetches the shared Tiger SVG and parses it once into a GraphicsContext
//   - drives a manual render loop (we call renderer.render() ourselves so we
//     can time it, instead of using app.ticker which hides the cost)
//   - mirrors arabella's overlay: FPS, CPU ms, GPU ms, zoom, ops
//   - wires the same pan/zoom/reset controls
//
// IMPORTANT measurement note:
//   For a fair comparison with arabella + vello_hybrid, we time the
//   *synchronous* CPU cost of building + submitting the frame. PixiJS's
//   renderer.render() is synchronous on the CPU (it tessellates dirty
//   Graphics, builds batches, and issues GL/Canvas2D draw calls). It does
//   NOT block on GPU completion, exactly like arabella's and vello's "GPU"
//   number measures submission, not GPU compute. So we report one combined
//   "CPU" number for PixiJS (build + submit) and leave GPU as n/a, because
//   PixiJS doesn't expose a separate submit phase we can isolate cleanly.

const PIXI_CDN = 'https://cdn.jsdelivr.net/npm/pixi.js@8.18.1/dist/pixi.min.mjs';

// Path to the shared Tiger asset, relative to each demo's index.html
// (which lives one directory deeper than this shared/ folder).
const TIGER_SVG_URL = '../shared/Ghostscript_Tiger.svg';

/**
 * @param {object} opts
 * @param {'canvas'|'webgl'} opts.preference  PixiJS renderer preference.
 * @param {string} opts.label                 Display name in the overlay.
 */
export async function runBenchmark(opts) {
    const { preference, label } = opts;

    // ── Load PixiJS from CDN ──
    const PIXI = await import(PIXI_CDN);
    const { Application, Graphics, GraphicsContext, Container } = PIXI;

    // ── Page chrome ──
    document.body.style.margin = '0';
    document.body.style.overflow = 'hidden';
    document.body.style.background = '#111';

    const dpr = window.devicePixelRatio || 1;
    const cssW = window.innerWidth;
    const cssH = window.innerHeight;

    // ── Create the application with the requested backend ──
    // `preference: ['canvas']` (array) forces ONLY the canvas renderer —
    // no webgpu/webgl fallback — so we know exactly which backend we're
    // measuring. Same for ['webgl'].
    const app = new Application();
    await app.init({
        preference: [preference],
        width: cssW,
        height: cssH,
        backgroundColor: 0x111111,
        antialias: true,
        resolution: dpr,
        autoDensity: true,
        // We drive rendering manually so we can time renderer.render().
        autoStart: false,
    });
    document.body.appendChild(app.canvas);
    app.canvas.style.display = 'block';
    app.canvas.style.touchAction = 'none';

    // The actual backend that got selected (e.g. "canvas" / "webgl").
    const backendName = app.renderer.name;

    // ── Build the Tiger once into a shared GraphicsContext ──
    const svgText = await fetch(TIGER_SVG_URL).then((r) => r.text());

    // Parsing SVG → GraphicsContext is the expensive one-time step. We do
    // it once and reuse the context for the Graphics object, so per-frame
    // cost is purely transform + tessellate/raster + submit (matching how
    // arabella reuses its parsed paint-ops).
    const ctx = new GraphicsContext().svg(svgText);
    const tiger = new Graphics(ctx);

    // Center + scale to roughly fill, mirroring arabella's framing.
    const camera = new Container();
    camera.addChild(tiger);
    app.stage.addChild(camera);

    // Frame the tiger: compute local bounds and recenter.
    const bounds = tiger.getLocalBounds();
    const tigerW = bounds.width;
    const tigerH = bounds.height;
    const fitScale = Math.min(cssW / tigerW, cssH / tigerH) * 0.9;

    // View state — applied to the `camera` container.
    const view = {
        x: cssW / 2,
        y: cssH / 2,
        scale: fitScale,
    };
    // Pivot the tiger around its own center so scaling zooms about center.
    tiger.pivot.set(bounds.x + tigerW / 2, bounds.y + tigerH / 2);

    function applyView() {
        camera.position.set(view.x, view.y);
        camera.scale.set(view.scale);
    }
    applyView();

    // Count of subpaths/ops for the overlay (best-effort; PixiJS doesn't
    // expose a clean op count, so we count SVG path elements).
    const opCount = (svgText.match(/<path/g) || []).length;

    // ── Overlay (same layout as arabella) ──
    const overlay = document.createElement('div');
    Object.assign(overlay.style, {
        position: 'fixed',
        top: '10px',
        left: '10px',
        padding: '8px 12px',
        background: 'rgba(0,0,0,0.6)',
        color: '#eee',
        fontFamily: 'ui-monospace, Menlo, Consolas, monospace',
        fontSize: '12px',
        lineHeight: '1.5',
        borderRadius: '6px',
        pointerEvents: 'none',
        zIndex: '10',
        whiteSpace: 'nowrap',
    });
    overlay.innerHTML = 'starting…';
    document.body.appendChild(overlay);

    // ── Timing state ──
    let lastFrameTime = 0;
    const fpsWindow = [];
    let lastCpuMs = 0;
    let frameCounter = 0;
    let needRender = true; // render at least the first frame

    function updateOverlay() {
        const avgDt =
            fpsWindow.length === 0
                ? 0
                : fpsWindow.reduce((a, b) => a + b, 0) / fpsWindow.length;
        const fps = avgDt > 0 ? 1000 / avgDt : 0;
        overlay.innerHTML =
            `<b>PixiJS</b> &nbsp; <span style="color:#9cf">${label}</span> ` +
            `<span style="color:#6c6">[${backendName}]</span><br/>` +
            `FPS: <b>${fps.toFixed(1).padStart(5)}</b> (${avgDt
                .toFixed(2)
                .padStart(5)} ms)<br/>` +
            `CPU+submit: ${lastCpuMs.toFixed(2).padStart(5)} ms<br/>` +
            `zoom: ${view.scale / fitScale > 0 ? (view.scale / fitScale).toFixed(2) : '?'}× &nbsp; ops: ${opCount}<br/>` +
            `<span style="color:#aaa">drag pan · wheel zoom · space reset</span>`;
    }

    function renderFrame() {
        // We always re-render here (the tiger is static, but we want a
        // continuous FPS readout during interaction). To match arabella —
        // which re-bins every frame — we force PixiJS to re-tessellate
        // (Canvas2D: re-rasterize) every frame by setting the context's
        // `dirty` flag. GraphicsContextSystem rebuilds batches/geometry
        // whenever `context.dirty` is true, then clears it.
        //
        // Without this, PixiJS caches the tessellation and per-frame cost
        // collapses to a transform-only submit, which would NOT be
        // comparable to arabella's per-frame rebuild. See README.
        const perf = performance;

        const cpuStart = perf.now();
        ctx.dirty = true; // force re-tessellation / re-raster this frame
        app.renderer.render(app.stage);
        const cpuEnd = perf.now();

        lastCpuMs = cpuEnd - cpuStart;

        const now = perf.now();
        if (lastFrameTime > 0) {
            const dt = now - lastFrameTime;
            fpsWindow.push(dt);
            if (fpsWindow.length > 60) fpsWindow.shift();
        }
        lastFrameTime = now;

        frameCounter = (frameCounter + 1) % 10;
        if (frameCounter === 0) updateOverlay();
    }

    // Continuous rAF loop (matches arabella's loop; PixiJS autoStart is off).
    function loop() {
        renderFrame();
        requestAnimationFrame(loop);
    }
    requestAnimationFrame(loop);

    // Also log a cached-vs-uncached comparison once, for the thesis writeup.
    logCachedComparison(app, ctx);

    // ── Controls ──
    let mouseDown = false;
    let lastCursor = null;

    app.canvas.addEventListener('mousedown', (ev) => {
        ev.preventDefault();
        mouseDown = true;
        lastCursor = [ev.clientX, ev.clientY];
    });
    window.addEventListener('mouseup', () => {
        mouseDown = false;
    });
    app.canvas.addEventListener('mousemove', (ev) => {
        if (mouseDown && lastCursor) {
            const dx = ev.clientX - lastCursor[0];
            const dy = ev.clientY - lastCursor[1];
            // PixiJS screen space is y-down (same as the browser), so no
            // Y flip needed here — unlike arabella whose scene space is
            // y-up. Dragging moves content the natural way.
            view.x += dx;
            view.y += dy;
            applyView();
        }
        lastCursor = [ev.clientX, ev.clientY];
    });
    app.canvas.addEventListener(
        'wheel',
        (ev) => {
            ev.preventDefault();
            const ZOOM_STEP = 0.1;
            const factor = Math.max(1 + (-ev.deltaY / 100) * ZOOM_STEP, 0.1);

            // Zoom centered on the cursor: keep the world point under the
            // cursor fixed.  screen = view.pos + scale * (world - pivot)
            // Solve so the cursor's world point is invariant.
            const cx = ev.clientX;
            const cy = ev.clientY;
            view.x = cx + (view.x - cx) * factor;
            view.y = cy + (view.y - cy) * factor;
            view.scale *= factor;
            applyView();
        },
        { passive: false },
    );
    window.addEventListener('keydown', (ev) => {
        if (ev.key === ' ') {
            view.x = cssW / 2;
            view.y = cssH / 2;
            view.scale = fitScale;
            applyView();
            ev.preventDefault();
        }
    });

    // Resize.
    window.addEventListener('resize', () => {
        const w = window.innerWidth;
        const h = window.innerHeight;
        app.renderer.resize(w, h);
    });

    console.log(`[bench] PixiJS backend = ${backendName}, ops = ${opCount}`);
}

/**
 * Log a one-shot cached-vs-uncached timing comparison to the console. This
 * gives the thesis a fair "best case" (PixiJS caching the tessellation)
 * alongside the per-frame-rebuild number shown in the overlay.
 */
function logCachedComparison(app, ctx) {
    const N = 60;

    // Uncached: bump dirty every frame.
    let t0 = performance.now();
    for (let i = 0; i < N; i++) {
        ctx.dirty = true;
        app.renderer.render(app.stage);
    }
    let uncached = (performance.now() - t0) / N;

    // Cached: render the same (now-clean) graphics repeatedly.
    t0 = performance.now();
    for (let i = 0; i < N; i++) {
        app.renderer.render(app.stage);
    }
    let cached = (performance.now() - t0) / N;

    console.log(
        `[bench] avg render over ${N} frames — ` +
            `uncached(rebuild): ${uncached.toFixed(2)} ms, ` +
            `cached(reuse tess): ${cached.toFixed(2)} ms`,
    );
}
