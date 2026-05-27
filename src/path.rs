use crate::flatten::flatten_quadratic;
use alloc::{format, vec::Vec};
use fearless_simd::*;
use lyon_geom::{Box2D, Point, Transform};
use lyon_path::{path::Iter, PathEvent};

const MAX_QUADS: usize = 16;
const TO_QUAD_TOL: f32 = 0.5;
const EPSILON: f32 = 1e-6;
pub(crate) const SQRT_TOL: f32 = 0.5;
pub(crate) const TOL: f32 = SQRT_TOL * SQRT_TOL;

/// Maximum acceptable gap (in pixel units) between our running `last_pt`
/// and the `from` field of the next event. Anything beyond this means our
/// state machine has fallen out of sync with the path iterator and we'll
/// emit asymmetric crossings → rightward streaks.
const CONTINUITY_TOL: f32 = 1e-3;

// ============================================================================
// SIMD-batched affine point transform
//
// `transform_point` per call is `(m11*x + m21*y + m31, m12*x + m22*y + m32)`
// — 4 mults + 4 adds. Doing this lane-by-lane on 4 points (one Cubic event)
// is 16 mults + 16 adds. With f32x8 we collapse the four mul-add chains
// into two FMAs. `transform_pair` (2 points) is the common case for Line
// events and `transform_quad` covers Cubic.
//
// Layout for the 4-point case:
//   input  = [x0, y0, x1, y1, x2, y2, x3, y3]
//   m_xy   = [m11, m12, m11, m12, m11, m12, m11, m12]   (row 1 of 2×2)
//   m_yx   = [m21, m22, m21, m22, m21, m22, m21, m22]   (row 2 of 2×2)
//   t      = [m31, m32, m31, m32, m31, m32, m31, m32]
//   xs     = [x0, x0, x1, x1, x2, x2, x3, x3]            (broadcast x)
//   ys     = [y0, y0, y1, y1, y2, y2, y3, y3]            (broadcast y)
//   out    = xs*m_xy + ys*m_yx + t
//
// The "broadcast" requires shuffles which fearless_simd doesn't expose
// cleanly for 8-wide types. Instead, we construct `xs`/`ys` directly from
// the input scalars (compiler turns this into 2 loads or shuffles) — the
// FMA chain is what matters, not the construction. Net: ~half the issue
// rate of the scalar form, plus better register usage.
// ============================================================================

#[inline(always)]
fn transform_pair<S: Simd>(
    simd: S,
    affine: &Transform<f32>,
    p: Point<f32>,
    q: Point<f32>,
) -> (Point<f32>, Point<f32>) {
    // Affine in lyon is row-major with .m11..m32 fields:
    //   X = m11*x + m21*y + m31
    //   Y = m12*x + m22*y + m32
    let m_xy = f32x4::from_slice(simd, &[affine.m11, affine.m12, affine.m11, affine.m12]);
    let m_yx = f32x4::from_slice(simd, &[affine.m21, affine.m22, affine.m21, affine.m22]);
    let t    = f32x4::from_slice(simd, &[affine.m31, affine.m32, affine.m31, affine.m32]);

    let xs = f32x4::from_slice(simd, &[p.x, p.x, q.x, q.x]);
    let ys = f32x4::from_slice(simd, &[p.y, p.y, q.y, q.y]);

    // FMA chain: r = xs*m_xy + ys*m_yx + t
    let r = xs.mul_add(m_xy, ys.mul_add(m_yx, t));
    let s = r.as_slice();
    (Point::new(s[0], s[1]), Point::new(s[2], s[3]))
}

#[inline(always)]
fn transform_quad<S: Simd>(
    simd: S,
    affine: &Transform<f32>,
    p: Point<f32>,
    q: Point<f32>,
    r: Point<f32>,
    s: Point<f32>,
) -> (Point<f32>, Point<f32>, Point<f32>, Point<f32>) {
    let m_xy = f32x8::from_slice(simd, &[
        affine.m11, affine.m12, affine.m11, affine.m12,
        affine.m11, affine.m12, affine.m11, affine.m12,
    ]);
    let m_yx = f32x8::from_slice(simd, &[
        affine.m21, affine.m22, affine.m21, affine.m22,
        affine.m21, affine.m22, affine.m21, affine.m22,
    ]);
    let t = f32x8::from_slice(simd, &[
        affine.m31, affine.m32, affine.m31, affine.m32,
        affine.m31, affine.m32, affine.m31, affine.m32,
    ]);

    let xs = f32x8::from_slice(simd, &[p.x, p.x, q.x, q.x, r.x, r.x, s.x, s.x]);
    let ys = f32x8::from_slice(simd, &[p.y, p.y, q.y, q.y, r.y, r.y, s.y, s.y]);

    let out = xs.mul_add(m_xy, ys.mul_add(m_yx, t));
    let v = out.as_slice();
    (
        Point::new(v[0], v[1]),
        Point::new(v[2], v[3]),
        Point::new(v[4], v[5]),
        Point::new(v[6], v[7]),
    )
}



/// Compare lyon's reported `from` against our running `last_pt`.
/// Logs to the browser console when they disagree.
///
/// Compiled to a no-op in release builds because the check itself plus
/// the call site overhead (one comparison per event) is unnecessary in
/// the hot path. We keep it for debug builds where the diagnostic is
/// worth the cost.
#[cfg(debug_assertions)]
#[inline(always)]
fn check_continuity(kind: &'static str, last_pt: Point<f32>, from: Point<f32>) {
    let dx = (last_pt.x - from.x).abs();
    let dy = (last_pt.y - from.y).abs();
    if dx > CONTINUITY_TOL || dy > CONTINUITY_TOL {
        web_sys::console::log_1(
            &alloc::format!(
                "[CONTINUITY {} EVENT MISMATCH] last_pt=({:.4},{:.4}) from=({:.4},{:.4}) Δ=({:.4},{:.4})",
                kind, last_pt.x, last_pt.y, from.x, from.y, dx, dy,
            )
            .into(),
        );
    }
}

#[cfg(not(debug_assertions))]
#[inline(always)]
fn check_continuity(_kind: &'static str, _last_pt: Point<f32>, _from: Point<f32>) {}

// ============================================================================
// Public entry point
// ============================================================================

pub fn fill_impl<'a, S: Simd>(
    simd: S,
    path: Iter,
    affine: Transform<f32>,
    line_buf: &'a mut Vec<i32>,
    bbox: &mut Box2D<f32>,
) {
    // line_buf stores one flattened LINE per record: 4 i32s (F24Dot8) = [p0x, p0y, p1x, p1y].
    // No line_id — clipped endpoints are produced per-tile by the DDA in blocks.rs.
    let mut iter = path;

    let Some(first_el) = iter.next() else { return; };
    let PathEvent::Begin { at } = first_el else { return; };

    let at = affine.transform_point(at);
    let mut start_pt = at;
    let mut last_pt = start_pt;
    let mut counter = 0;
    expand_bbox(bbox, at);

    // Initial Path Entry Log
    // web_sys::console::log_1(&format!("[PATH START] Initialized sub-path at: {:?}", start_pt).into());

    for event in iter {
        match event {
            PathEvent::Begin { at } => {
                
                let at = affine.transform_point(at);
                
                // web_sys::console::log_1(&format!(
                //     "[PATH BEGIN] New Sub-Path Loop! Closing old? {}. Moving to: {:?}", 
                //     last_pt != start_pt, at
                // ).into());

                if last_pt != start_pt {
                    emit_line(line_buf, last_pt, start_pt);
                }
                start_pt = at;
                last_pt = at;
                expand_bbox(bbox, at);
              
            }
            PathEvent::Line { from, to } => {
                let (from_t, to) = transform_pair(simd, &affine, from, to);

                check_continuity("Line", last_pt, from_t);

                if (to.x - last_pt.x).abs() > EPSILON || (to.y - last_pt.y).abs() > EPSILON {
                    emit_line(line_buf, last_pt, to);
                    expand_bbox(bbox, to);
                    last_pt = to;
                }
            }
            PathEvent::Quadratic { from, ctrl, to } => {
                // Quadratic has 3 points; transform 2 + 1.
                let (from_t, ctrl) = transform_pair(simd, &affine, from, ctrl);
                let (to, _) = transform_pair(simd, &affine, to, to);

                check_continuity("Quadratic", last_pt, from_t);

                expand_bbox(bbox, ctrl);
                expand_bbox(bbox, to);

                if let Some(t) = find_quadratic_extrema(last_pt.y, ctrl.y, to.y) {
                    let ab = last_pt.lerp(ctrl, t);
                    let bc = ctrl.lerp(to, t);
                    let mid = ab.lerp(bc, t);
                    expand_bbox(bbox, mid);

                    emit_quadratic_and_flatten(line_buf, last_pt, ab, mid);
                    emit_quadratic_and_flatten(line_buf, mid, bc, to);
                } else {
                    emit_quadratic_and_flatten(line_buf, last_pt, ctrl, to);
                }
                last_pt = to;
            }
            PathEvent::Cubic { from, ctrl1, ctrl2, to } => {
                // Cubic has 4 points → fits one f32x8 fused transform.
                let (from_t, ctrl1, ctrl2, to) =
                    transform_quad(simd, &affine, from, ctrl1, ctrl2, to);

                check_continuity("Cubic", last_pt, from_t);

                expand_bbox(bbox, ctrl1);
                expand_bbox(bbox, ctrl2);
                expand_bbox(bbox, to);

                let lp = Point::new(last_pt.x, last_pt.y);
                let c1 = Point::new(ctrl1.x, ctrl1.y);
                let c2 = Point::new(ctrl2.x, ctrl2.y);
                let tp = Point::new(to.x, to.y);

                let n = estimate_number_of_quadratic_curves(&lp, &c1, &c2, &tp, TOL);
                let q = convert_cubics_to_quadratic_curves(simd, &lp, &c1, &c2, &tp, n);

                let mut curve_from = last_pt;

                for i in 0..n {
                    let ctrl = Point::new(q[2 + i * 4], q[2 + i * 4 + 1]);
                    let end = Point::new(q[2 + i * 4 + 2], q[2 + i * 4 + 3]);

                    // Sub-quadratic control points from cubic→quadratic conversion can
                    // extend OUTSIDE the original cubic's convex hull. Without expanding
                    // bbox to include them (and the y-extremum mid below), flattened
                    // lines whose endpoints fall outside the bbox-derived tile grid
                    // get silently dropped by the DDA's `>= covers.rows()` guard.
                    // That breaks winding cancellation and produces rightward streaks.
                    expand_bbox(bbox, ctrl);
                    expand_bbox(bbox, end);

                    if let Some(t) = find_quadratic_extrema(curve_from.y, ctrl.y, end.y) {
                        let ab = curve_from.lerp(ctrl, t);
                        let bc = ctrl.lerp(end, t);
                        let mid = ab.lerp(bc, t);
                        expand_bbox(bbox, mid);

                        emit_quadratic_and_flatten(line_buf, curve_from, ab, mid);
                        emit_quadratic_and_flatten(line_buf, mid, bc, end);
                    } else {
                        emit_quadratic_and_flatten(line_buf, curve_from, ctrl, end);
                    }
                    curve_from = end;
                }
                last_pt = to;
            }
            PathEvent::End { last, first, close } => {
                let (last, first) = transform_pair(simd, &affine, last, first);


                if close && last != first {
                    emit_line(line_buf, last, first);
                }

                // CRITICAL: keep last_pt consistent with what the closing
                // line just did. Otherwise the next `Begin` (or the
                // end-of-loop fallback) compares stale `last_pt` against
                // the current `start_pt` and emits the SAME closing line
                // again, doubling its winding contribution on the closing-
                // edge scanlines and leaving a rightward streak.
                if close {
                    last_pt = first;
                } else {
                    last_pt = last;
                }
            }
        }
    }

    if last_pt != start_pt {
        emit_line(line_buf, last_pt, start_pt);
    }
}

/// Flatten a quadratic into straight lines, pushing each into `line_buf`
/// as 4 i32s (F24Dot8): `[p0x, p0y, p1x, p1y]`.
#[inline(always)]
fn emit_quadratic_and_flatten(
    line_buf: &mut Vec<i32>,
    p0: Point<f32>,
    p1: Point<f32>,
    p2: Point<f32>,
) {
    flatten_quadratic(
        f32_to_f24dot8(p0.x), f32_to_f24dot8(p0.y),
        f32_to_f24dot8(p1.x), f32_to_f24dot8(p1.y),
        f32_to_f24dot8(p2.x), f32_to_f24dot8(p2.y),
        &mut |x0, y0, x1, y1| {
            line_buf.extend_from_slice(&[x0, y0, x1, y1]);
        },
    );
}

// ============================================================================
// Helpers
// ============================================================================

/// Push a single straight line as 4 i32s (F24Dot8): `[p0x, p0y, p1x, p1y]`.
#[inline(always)]
fn emit_line(
    line_buf: &mut Vec<i32>,
    from: Point<f32>,
    to: Point<f32>,
) {
    line_buf.extend_from_slice(&[
        f32_to_f24dot8(from.x), f32_to_f24dot8(from.y),
        f32_to_f24dot8(to.x),   f32_to_f24dot8(to.y),
    ]);
}

#[inline(always)]
fn expand_bbox(bbox: &mut Box2D<f32>, pt: Point<f32>) {
    bbox.min.x = bbox.min.x.min(pt.x);
    bbox.min.y = bbox.min.y.min(pt.y);
    bbox.max.x = bbox.max.x.max(pt.x);
    bbox.max.y = bbox.max.y.max(pt.y);
}

#[inline(always)]
fn f32_to_f24dot8(v: f32) -> i32 {
    // Symmetric round-to-nearest. The previous `(v * 256.0 + 0.5) as i32`
    // works only for positive values: for negatives, `as i32` truncates
    // toward zero instead of rounding to nearest, which biases winding
    // asymmetrically and leaves residual scanline accumulators that leak
    // rightward as visible streaks.
    let scaled = v * 256.0;
    if scaled >= 0.0 {
        (scaled + 0.5) as i32
    } else {
        (scaled - 0.5) as i32
    }
}

// ============================================================================
// Cubic → Quadratic conversion helpers
// ============================================================================

#[inline(always)]
fn estimate_number_of_quadratic_curves(
    p0: &Point<f32>,
    p1: &Point<f32>,
    p2: &Point<f32>,
    p3: &Point<f32>,
    accuracy: f32,
) -> usize {
    let q_accuracy = accuracy * TO_QUAD_TOL;
    let max_hypot2 = 432.0 * q_accuracy * q_accuracy;

    let p1x2_x = p1.x * 3.0 - p0.x;
    let p1x2_y = p1.y * 3.0 - p0.y;
    let p2x2_x = p2.x * 3.0 - p3.x;
    let p2x2_y = p2.y * 3.0 - p3.y;

    let dx = p2x2_x - p1x2_x;
    let dy = p2x2_y - p1x2_y;
    let err = dx * dx + dy * dy;

    let err_div = err / max_hypot2;

    estimate(err_div as f64)
}

#[inline(always)]
fn estimate(err_div: f64) -> usize {
    const LUT: [f64; MAX_QUADS] = [
        1.0, 64.0, 729.0, 4096.0, 15625.0, 46656.0, 117649.0, 262144.0,
        531441.0, 1000000.0, 1771561.0, 2985984.0, 4826809.0, 7529536.0,
        11390625.0, 16777216.0,
    ];
    for i in 0..MAX_QUADS {
        if err_div <= LUT[i] {
            return i + 1;
        }
    }
    MAX_QUADS
}

fn convert_cubics_to_quadratic_curves<S: Simd>(
    simd: S,
    p0: &Point<f32>,
    p1: &Point<f32>,
    p2: &Point<f32>,
    p3: &Point<f32>,
    n: usize,
) -> [f32; 96] {
    let dt = 0.5 / n as f32;

    #[inline(always)]
    fn splat_point_xy<S: Simd>(simd: S, x: f32, y: f32) -> f32x8<S> {
        f32x8::from_slice(simd, &[x, y, x, y, x, y, x, y])
    }

    let p0v = splat_point_xy(simd, p0.x, p0.y);
    let p1v = splat_point_xy(simd, p1.x, p1.y);
    let p2v = splat_point_xy(simd, p2.x, p2.y);
    let p3v = splat_point_xy(simd, p3.x, p3.y);

    let coeff_a = (p1v - p2v).mul_add(3.0, p3v - p0v);
    let coeff_b = p1v.mul_add(-2.0, p0v + p2v) * 3.0;
    let coeff_c = (p1v - p0v) * 3.0;
    let coeff_d = p0v;

    let lane_iota = f32x8::from_slice(simd, &[0.0, 0.0, 2.0, 2.0, 1.0, 1.0, 3.0, 3.0]);
    let mut t = lane_iota * dt;
    let t_inc = f32x8::splat(simd, 4.0 * dt);

    let mut even_pts = [[0.0_f32; 2]; MAX_QUADS + 4];
    let mut odd_pts = [[0.0_f32; 2]; MAX_QUADS];

    for i in 0..n.div_ceil(2) {
        let evaluated = (coeff_a.mul_add(t, coeff_b))
            .mul_add(t, coeff_c)
            .mul_add(t, coeff_d);

        let (low, high) = simd.split_f32x8(evaluated);
        let lo = low.as_slice();
        let hi = high.as_slice();

        let e = i * 2;
        if e < (n + 1) {
            even_pts[e] = [lo[0], lo[1]];
        }
        if e + 1 < (n + 1) {
            even_pts[e + 1] = [lo[2], lo[3]];
        }

        let o = i * 2;
        if o < n {
            odd_pts[o] = [hi[0], hi[1]];
        }
        if o + 1 < n {
            odd_pts[o + 1] = [hi[2], hi[3]];
        }

        t += t_inc;
    }

    even_pts[n] = [p3.x, p3.y];

    // Compute control points
    let mut ctrl_pts = [[0.0_f32; 2]; MAX_QUADS];

    for i in 0..n.div_ceil(4) {
        let p0v = f32x8::from_slice(simd, &even_pts.as_flattened()[i * 8..][..8]);
        let p_onehalf = f32x8::from_slice(simd, &odd_pts.as_flattened()[i * 8..][..8]);
        let p2v = f32x8::from_slice(simd, &even_pts.as_flattened()[(i * 8 + 2)..][..8]);

        let x = p0v * -0.5;
        let x1 = p_onehalf.mul_add(2.0, x);
        let p1v = p2v.mul_add(-0.5, x1);

        let p1_slice = p1v.as_slice();
        for j in 0..4 {
            let k = i * 4 + j;
            if k < n {
                ctrl_pts[k] = [p1_slice[j * 2], p1_slice[j * 2 + 1]];
            }
        }
    }

    // Write continuous stream:
    // [P0.x, P0.y, ctrl0.x, ctrl0.y, P1.x, P1.y, ctrl1.x, ctrl1.y, P2.x, P2.y, ...]
    // Layout: first 2 floats = initial on-curve point
    //         then for each curve: 4 floats = (ctrl.x, ctrl.y, endpoint.x, endpoint.y)
    let mut out: [f32; 96] = [0.0; 96];
    let mut index = 0;

    // First on-curve point
    out[index] = even_pts[0][0];
    out[index + 1] = even_pts[0][1];
    index += 2;

    // For each sub-quadratic: ctrl + endpoint
    for k in 0..n {
        out[index] = ctrl_pts[k][0];
        out[index + 1] = ctrl_pts[k][1];
        out[index + 2] = even_pts[k + 1][0];
        out[index + 3] = even_pts[k + 1][1];
        index += 4;
    }

    out
}

fn find_quadratic_extrema(a: f32, b: f32, c: f32) -> Option<f32> {
    let a_min_b = a - b;
    let d = a_min_b - b + c;

    if a_min_b == 0.0 || d == 0.0 {
        return None;
    }

    let t = a_min_b / d;

    if t <= 1e-6 || t >= (1.0 - 1e-6) {
        return None;
    }
    Some(t)
}
