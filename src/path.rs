use crate::flatten::flatten_quadratic;
use alloc::vec::Vec;
use bytemuck::{Pod, Zeroable};
use fearless_simd::*;
use lyon_geom::{Box2D, CubicBezierSegment, LineSegment, Point, QuadraticBezierSegment, Transform};
use lyon_path::{Event, Iter, Path, PathEvent, polygon::PathEvents};
const MAX_QUADS: usize = 16;
const TO_QUAD_TOL: f32 = 0.5;
const EPSILON: f32 = 1e-6;
pub(crate) const SQRT_TOL: f32 = 0.5;
pub(crate) const TOL: f32 = SQRT_TOL * SQRT_TOL;

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
        1.0, 64.0, 729.0, 4096.0, 15625.0, 46656.0, 117649.0, 262144.0, 531441.0, 1000000.0,
        1771561.0, 2985984.0, 4826809.0, 7529536.0, 11390625.0, 16777216.0,
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

    let mut out: [f32; 96] = [0.0; 96];

    // Continuous stream: P0, [P1, P2] × n
    // Total floats = 2 + n*4, where P2 of curve k == P0 of curve k+1 (written once)
    //
    // Layout: [ap0.x, ap0.y, ap1.x, ap1.y, ap2.x, ap2.y, bp1.x, bp1.y, bp2.x, bp2.y, ...]
    //                                        ^ shared, not repeated

    // Compute all control points p1[0..n] into a temp buffer first
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

    // Now write the continuous stream: P0, then for each curve: ctrl, endpoint
    let mut index = 0;
    out[index] = even_pts[0][0]; // ap0.x
    out[index + 1] = even_pts[0][1]; // ap0.y
    index += 2;

    for k in 0..n {
        out[index] = ctrl_pts[k][0]; // Pk_1 (control point)
        out[index + 1] = ctrl_pts[k][1];
        out[index + 2] = even_pts[k + 1][0]; // Pk_2 == P(k+1)_0 (shared endpoint)
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

    if t <= 1e-12 || t >= (1.0 - 1e-12) {
        return None;
    }
    Some(t)
}
pub fn fill_impl<'a, S: Simd>(
    simd: S,
    path: Iter,
    affine: Transform<f32>,
    segments: &'a mut Vec<f32>,
    line_buf: &'a mut Vec<i32>,
    bbox: &mut Box2D<f32>,
) {
    let mut curve_id = (segments.len() / 8) as i32;
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
                if last_pt != start_pt {
                    // Close previous subpath with a line
                    let mid_x = (last_pt.x + start_pt.x) * 0.5;
                    let mid_y = (last_pt.y + start_pt.y) * 0.5;
                    let line_curve_id = curve_id;
                    curve_id += 1;
                    segments.extend_from_slice(&[
                        last_pt.x, last_pt.y, mid_x, mid_y, start_pt.x, start_pt.y, 0.0, 0.0,
                    ]);
                    line_buf.extend_from_slice(&[
                        f32_to_f24dot8(last_pt.x),
                        f32_to_f24dot8(last_pt.y),
                        f32_to_f24dot8(start_pt.x),
                        f32_to_f24dot8(start_pt.y),
                        line_curve_id,
                    ]);
                }
                start_pt = at;
                last_pt = at;
                expand_bbox(bbox, at);
            }
            PathEvent::Line { from: _, to } => {
                let to = affine.transform_point(to);
                if (to.x - last_pt.x).abs() > EPSILON || (to.y - last_pt.y).abs() > EPSILON {
                    let mid_x = (last_pt.x + to.x) * 0.5;
                    let mid_y = (last_pt.y + to.y) * 0.5;
                    let line_curve_id = curve_id;
                    curve_id += 1;
                    segments.extend_from_slice(&[
                        last_pt.x, last_pt.y, mid_x, mid_y, to.x, to.y, 0.0, 0.0,
                    ]);
                    line_buf.extend_from_slice(&[
                        f32_to_f24dot8(last_pt.x),
                        f32_to_f24dot8(last_pt.y),
                        f32_to_f24dot8(to.x),
                        f32_to_f24dot8(to.y),
                        line_curve_id,
                    ]);
                    expand_bbox(bbox, to);
                    last_pt = to;
                }
            }
            PathEvent::Quadratic { from: _, ctrl, to } => {
                let ctrl = affine.transform_point(ctrl);
                let to = affine.transform_point(to);

                expand_bbox(bbox, ctrl);
                expand_bbox(bbox, to);

                if let Some(t) = find_quadratic_extrema(last_pt.y, ctrl.y, to.y) {
                    let ab = last_pt.lerp(ctrl, t);
                    let bc = ctrl.lerp(to, t);
                    let mid = ab.lerp(bc, t);
                    expand_bbox(bbox, mid);

                    // First half: from=last_pt, ctrl=ab, to=mid
                    segments.extend_from_slice(&[
                        last_pt.x, last_pt.y, ab.x, ab.y, mid.x, mid.y, 0.0, 0.0,
                    ]);
                    // Second half: from=mid, ctrl=bc, to=to
                    let first_curve_id = curve_id;
                    curve_id += 1;
                    segments.extend_from_slice(&[mid.x, mid.y, bc.x, bc.y, to.x, to.y, 0.0, 0.0]);
                    let second_curve_id = curve_id;
                    curve_id += 1;
                    flatten_quadratic(
                        f32_to_f24dot8(last_pt.x),
                        f32_to_f24dot8(last_pt.y),
                        f32_to_f24dot8(ab.x),
                        f32_to_f24dot8(ab.y),
                        f32_to_f24dot8(mid.x),
                        f32_to_f24dot8(mid.y),
                        &mut |x0, y0, x1, y1| {
                            line_buf.extend_from_slice(&[x0, y0, x1, y1, first_curve_id]);
                        },
                    );
                    flatten_quadratic(
                        f32_to_f24dot8(mid.x),
                        f32_to_f24dot8(mid.y),
                        f32_to_f24dot8(bc.x),
                        f32_to_f24dot8(bc.y),
                        f32_to_f24dot8(to.x),
                        f32_to_f24dot8(to.y),
                        &mut |x0, y0, x1, y1| {
                            line_buf.extend_from_slice(&[x0, y0, x1, y1, second_curve_id]);
                        },
                    );
                } else {
                    // Single curve: from=last_pt, ctrl=ctrl, to=to
                    segments.extend_from_slice(&[
                        last_pt.x, last_pt.y, ctrl.x, ctrl.y, to.x, to.y, 0.0, 0.0,
                    ]);
                    let this_curve_id = curve_id;
                    curve_id += 1;
                    flatten_quadratic(
                        f32_to_f24dot8(last_pt.x),
                        f32_to_f24dot8(last_pt.y),
                        f32_to_f24dot8(ctrl.x),
                        f32_to_f24dot8(ctrl.y),
                        f32_to_f24dot8(to.x),
                        f32_to_f24dot8(to.y),
                        &mut |x0, y0, x1, y1| {
                            line_buf.extend_from_slice(&[x0, y0, x1, y1, this_curve_id]);
                        },
                    );
                }
                last_pt = to;
            }
            PathEvent::Cubic {
                from: _,
                ctrl1,
                ctrl2,
                to,
            } => {
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

                let mut flat_from_x = f32_to_f24dot8(last_pt.x);
                let mut flat_from_y = f32_to_f24dot8(last_pt.y);

                // Track the "from" point for each sub-quadratic
                let mut curve_from_x = last_pt.x;
                let mut curve_from_y = last_pt.y;

                for i in 0..n {
                    let p1x = q[i * 4 + 2];
                    let p1y = q[i * 4 + 3];
                    let p2x = if i + 1 < n { q[(i + 1) * 4] } else { to.x };
                    let p2y = if i + 1 < n { q[(i + 1) * 4 + 1] } else { to.y };

                    if let Some(t) = find_quadratic_extrema(curve_from_y, p1y, p2y) {
                        let p0 = Point::new(curve_from_x, curve_from_y);
                        let p1 = Point::new(p1x, p1y);
                        let p2 = Point::new(p2x, p2y);
                        let ab = p0.lerp(p1, t);
                        let bc = p1.lerp(p2, t);
                        let mid = ab.lerp(bc, t);

                        // First half
                        segments.extend_from_slice(&[
                            curve_from_x,
                            curve_from_y,
                            ab.x,
                            ab.y,
                            mid.x,
                            mid.y,
                            0.0,
                            0.0,
                        ]);
                        // Second half
                        let first_curve_id = curve_id;
                        curve_id += 1;
                        segments
                            .extend_from_slice(&[mid.x, mid.y, bc.x, bc.y, p2.x, p2.y, 0.0, 0.0]);
                        let second_curve_id = curve_id;
                        curve_id += 1;
                        let mid_fx = f32_to_f24dot8(mid.x);
                        let mid_fy = f32_to_f24dot8(mid.y);

                        flatten_quadratic(
                            flat_from_x,
                            flat_from_y,
                            f32_to_f24dot8(ab.x),
                            f32_to_f24dot8(ab.y),
                            mid_fx,
                            mid_fy,
                            &mut |x0, y0, x1, y1| {
                                line_buf.extend_from_slice(&[x0, y0, x1, y1, first_curve_id]);
                            },
                        );
                        flatten_quadratic(
                            mid_fx,
                            mid_fy,
                            f32_to_f24dot8(bc.x),
                            f32_to_f24dot8(bc.y),
                            f32_to_f24dot8(p2.x),
                            f32_to_f24dot8(p2.y),
                            &mut |x0, y0, x1, y1| {
                                line_buf.extend_from_slice(&[x0, y0, x1, y1, second_curve_id]);
                            },
                        );

                        flat_from_x = f32_to_f24dot8(p2.x);
                        flat_from_y = f32_to_f24dot8(p2.y);
                    } else {
                        // Single curve
                        segments.extend_from_slice(&[
                            curve_from_x,
                            curve_from_y,
                            p1x,
                            p1y,
                            p2x,
                            p2y,
                            0.0,
                            0.0,
                        ]);
                        let this_curve_id = curve_id;
                        curve_id += 1;
                        let p2_fx = f32_to_f24dot8(p2x);
                        let p2_fy = f32_to_f24dot8(p2y);

                        flatten_quadratic(
                            flat_from_x,
                            flat_from_y,
                            f32_to_f24dot8(p1x),
                            f32_to_f24dot8(p1y),
                            p2_fx,
                            p2_fy,
                            &mut |x0, y0, x1, y1| {
                                line_buf.extend_from_slice(&[x0, y0, x1, y1]);
                            },
                        );

                        flat_from_x = p2_fx;
                        flat_from_y = p2_fy;
                    }

                    // Advance "from" for next sub-quadratic
                    curve_from_x = p2x;
                    curve_from_y = p2y;
                }
                last_pt = to;
            }
            PathEvent::End { last, first, close } => {
                let last = affine.transform_point(last);
                let first = affine.transform_point(first);
                if close && last != first {
                    let mid_x = (last.x + first.x) * 0.5;
                    let mid_y = (last.y + first.y) * 0.5;
                    let line_curve_id = curve_id;
                    curve_id += 1;
                    segments.extend_from_slice(&[
                        last.x, last.y, mid_x, mid_y, first.x, first.y, 0.0, 0.0,
                    ]);
                    line_buf.extend_from_slice(&[
                        f32_to_f24dot8(last.x),
                        f32_to_f24dot8(last.y),
                        f32_to_f24dot8(first.x),
                        f32_to_f24dot8(first.y),
                        line_curve_id,
                    ]);
                }
            }
        }
    }

    // Close final subpath
    if last_pt != start_pt {
        let mid_x = (last_pt.x + start_pt.x) * 0.5;
        let mid_y = (last_pt.y + start_pt.y) * 0.5;
        let line_curve_id = curve_id;
        curve_id += 1;
        segments.extend_from_slice(&[
            last_pt.x, last_pt.y, mid_x, mid_y, start_pt.x, start_pt.y, 0.0, 0.0,
        ]);
        line_buf.extend_from_slice(&[
            f32_to_f24dot8(last_pt.x),
            f32_to_f24dot8(last_pt.y),
            f32_to_f24dot8(start_pt.x),
            f32_to_f24dot8(start_pt.y),
            line_curve_id,
        ]);
    }

    // Close final subpath
    if last_pt != start_pt {
        let mid_x = (last_pt.x + start_pt.x) * 0.5;
        let mid_y = (last_pt.y + start_pt.y) * 0.5;
        segments.extend_from_slice(&[
            last_pt.x, last_pt.y, mid_x, mid_y, start_pt.x, start_pt.y, 0.0, 0.0,
        ]);
        line_buf.extend_from_slice(&[
            f32_to_f24dot8(last_pt.x),
            f32_to_f24dot8(last_pt.y),
            f32_to_f24dot8(start_pt.x),
            f32_to_f24dot8(start_pt.y),
        ]);
    }
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
    (v * 256.0 + 0.5) as i32
}
