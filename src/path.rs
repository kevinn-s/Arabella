use alloc::vec::Vec;
use fearless_simd::*;
use lyon_geom::{Box2D, Point, Transform};
use lyon_path::{path::Iter, PathEvent};

const MAX_QUADS: usize = 16;
const TO_QUAD_TOL: f32 = 0.5;
const EPSILON: f32 = 1e-6;
pub(crate) const SQRT_TOL: f32 = 0.5;
pub(crate) const TOL: f32 = SQRT_TOL * SQRT_TOL;

// ============================================================================
// Curve record: one quadratic bezier stored as 8 floats in `segments`.
//
// Layout per curve (8 floats = 2 RGBA32F texels on GPU):
//   [p0.x, p0.y, p1.x, p1.y, p2.x, p2.y, 0.0, 0.0]
//
// Where p0 = start, p1 = control, p2 = end.
//
// Lines are stored as degenerate quadratics with p1 = midpoint(p0, p2).
// No y-monotonization — the shader handles non-monotonic curves via
// root-finding (GLLabel/Dobbie approach).
// ============================================================================

/// Process a path into quadratic bezier segments for GPU evaluation.
///
/// # Output
/// - `segments`: flat array of f32, 8 floats per curve (p0, ctrl, p2, pad).
///   curve_id = index / 8 (i.e. the Nth group of 8 floats is curve N).
/// - `bbox`: bounding box of all control points (expanded).
///
/// No line_buf, no flattening, no y-monotonization.
pub fn fill_impl<'a, S: Simd>(
    simd: S,
    path: Iter,
    affine: Transform<f32>,
    segments: &'a mut Vec<f32>,
    bbox: &mut Box2D<f32>,
) {
    let mut iter = path;

    let Some(first_el) = iter.next() else {
        return;
    };

    let PathEvent::Begin { at } = first_el else {
        return;
    };

    let at = affine.transform_point(at);
    let mut start_pt = at;
    let mut last_pt = start_pt;
    expand_bbox(bbox, at);

    for event in iter {
        match event {
            PathEvent::Begin { at } => {
                let at = affine.transform_point(at);
                // Close previous subpath
                if last_pt != start_pt {
                    emit_line(segments, last_pt, start_pt);
                }
                start_pt = at;
                last_pt = at;
                expand_bbox(bbox, at);
            }
            PathEvent::Line { from: _, to } => {
                let to = affine.transform_point(to);
                if (to.x - last_pt.x).abs() > EPSILON || (to.y - last_pt.y).abs() > EPSILON {
                    emit_line(segments, last_pt, to);
                    expand_bbox(bbox, to);
                    last_pt = to;
                }
            }
            PathEvent::Quadratic { from: _, ctrl, to } => {
                let ctrl = affine.transform_point(ctrl);
                let to = affine.transform_point(to);

                expand_bbox(bbox, ctrl);
                expand_bbox(bbox, to);

                // Store the quadratic as-is. No y-monotonization.
                // The shader's root-finding handles non-monotonic curves.
                emit_quad(segments, last_pt, ctrl, to);
                last_pt = to;
            }
            PathEvent::Cubic { from: _, ctrl1, ctrl2, to } => {
                let ctrl1 = affine.transform_point(ctrl1);
                let ctrl2 = affine.transform_point(ctrl2);
                let to = affine.transform_point(to);

                expand_bbox(bbox, ctrl1);
                expand_bbox(bbox, ctrl2);
                expand_bbox(bbox, to);

                let lp = Point::new(last_pt.x, last_pt.y);
                let c1 = Point::new(ctrl1.x, ctrl1.y);
                let c2 = Point::new(ctrl2.x, ctrl2.y);
                let tp = Point::new(to.x, to.y);

                let n = estimate_number_of_quadratic_curves(&lp, &c1, &c2, &tp, TOL);
                let q = convert_cubics_to_quadratic_curves(simd, &lp, &c1, &c2, &tp, n);

                // Output stream layout:
                //   q[0], q[1] = first on-curve point (P0)
                //   For each curve i (0..n):
                //     q[2 + i*4 + 0], q[2 + i*4 + 1] = control point
                //     q[2 + i*4 + 2], q[2 + i*4 + 3] = endpoint

                let mut from_x = last_pt.x;
                let mut from_y = last_pt.y;

                for i in 0..n {
                    let ctrl_x = q[2 + i * 4];
                    let ctrl_y = q[2 + i * 4 + 1];
                    let end_x = q[2 + i * 4 + 2];
                    let end_y = q[2 + i * 4 + 3];

                    // Expand bbox for the control point (endpoints already covered)
                    expand_bbox(bbox, Point::new(ctrl_x, ctrl_y));

                    // Emit quadratic directly — no y-monotonize, no flatten
                    segments.extend_from_slice(&[
                        from_x, from_y, ctrl_x, ctrl_y, end_x, end_y, 0.0, 0.0,
                    ]);

                    from_x = end_x;
                    from_y = end_y;
                }

                last_pt = to;
            }
            PathEvent::End { last, first, close } => {
                let last = affine.transform_point(last);
                let first = affine.transform_point(first);
                if close && last != first {
                    emit_line(segments, last, first);
                }
            }
        }
    }

    // Close final subpath if needed
    if last_pt != start_pt {
        emit_line(segments, last_pt, start_pt);
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Emit a line as a degenerate quadratic (ctrl = midpoint of endpoints).
#[inline(always)]
fn emit_line(segments: &mut Vec<f32>, from: Point<f32>, to: Point<f32>) {
    let mid_x = (from.x + to.x) * 0.5;
    let mid_y = (from.y + to.y) * 0.5;
    segments.extend_from_slice(&[from.x, from.y, mid_x, mid_y, to.x, to.y, 0.0, 0.0]);
}

/// Emit a quadratic bezier curve.
#[inline(always)]
fn emit_quad(segments: &mut Vec<f32>, from: Point<f32>, ctrl: Point<f32>, to: Point<f32>) {
    segments.extend_from_slice(&[from.x, from.y, ctrl.x, ctrl.y, to.x, to.y, 0.0, 0.0]);
}

#[inline(always)]
fn expand_bbox(bbox: &mut Box2D<f32>, pt: Point<f32>) {
    bbox.min.x = bbox.min.x.min(pt.x);
    bbox.min.y = bbox.min.y.min(pt.y);
    bbox.max.x = bbox.max.x.max(pt.x);
    bbox.max.y = bbox.max.y.max(pt.y);
}

// ============================================================================
// Cubic → Quadratic conversion (kept from original — this is correct)
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

    // Output stream: [P0.x, P0.y, ctrl0.x, ctrl0.y, end0.x, end0.y, ctrl1.x, ...]
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
