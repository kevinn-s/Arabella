use bytemuck::{Pod, Zeroable};
use fearless_simd::*;
use kurbo::{Affine, CubicBez, Line, ParamCurve, ParamCurveNearest, PathEl, Point, QuadBez, Shape};

use std::{cell::RefCell, f32, rc::Rc};

use rayon::prelude::*;

const MAX_QUADS: usize = 16;
const TO_QUAD_TOL: f32 = 16.0;
const EPSILON: f32 = 1e-12;
pub struct PathTag(pub u8);

impl PathTag {
    /// 32-bit floating point line segment.
    ///
    /// This is equivalent to `(PathSegmentType::LINE_TO | PathTag::F32_BIT)`.
    pub const LINE_TO_F32: Self = Self(0x9);

    /// 32-bit floating point quadratic segment.
    ///
    /// This is equivalent to `(PathSegmentType::QUAD_TO | PathTag::F32_BIT)`.
    pub const QUAD_TO_F32: Self = Self(0xa);

    /// 32-bit floating point cubic segment.
    ///
    /// This is equivalent to `(PathSegmentType::CUBIC_TO | PathTag::F32_BIT)`.
    pub const CUBIC_TO_F32: Self = Self(0xb);

    /// 16-bit integral line segment.
    pub const LINE_TO_I16: Self = Self(0x1);

    /// 16-bit integral quadratic segment.
    pub const QUAD_TO_I16: Self = Self(0x2);

    /// 16-bit integral cubic segment.
    pub const CUBIC_TO_I16: Self = Self(0x3);

    /// Transform marker.
    pub const TRANSFORM: Self = Self(0x20);

    /// Path marker.
    pub const PATH: Self = Self(0x10);

    /// Style setting.
    pub const STYLE: Self = Self(0x40);

    /// Bit that marks a segment that is the end of a subpath.
    pub const SUBPATH_END_BIT: u8 = 0x4;

    /// Bit for path segments that are represented as f32 values. If unset
    /// they are represented as i16.
    const F32_BIT: u8 = 0x8;

    /// Mask for bottom 3 bits that contain the [`PathSegmentType`].
    const SEGMENT_MASK: u8 = 0x3;

    /// Returns `true` if the tag is a segment.
    pub fn is_path_segment(self) -> bool {
        self.path_segment_type().0 != 0
    }

    /// Returns `true` if this is a 32-bit floating point segment.
    pub fn is_f32(self) -> bool {
        self.0 & Self::F32_BIT != 0
    }

    /// Returns `true` if this segment ends a subpath.
    pub fn is_subpath_end(self) -> bool {
        self.0 & Self::SUBPATH_END_BIT != 0
    }

    /// Sets the subpath end bit.
    pub fn set_subpath_end(&mut self) {
        self.0 |= Self::SUBPATH_END_BIT;
    }

    /// Returns the segment type.
    pub fn path_segment_type(self) -> PathSegmentType {
        PathSegmentType(self.0 & Self::SEGMENT_MASK)
    }
}

/// Path segment type.
///
/// The values of the segment types are equivalent to the number of associated
/// segments for each segment in the path segments stream.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Pod, Zeroable)]
#[repr(C)]
pub struct PathSegmentType(pub u8);

impl PathSegmentType {
    /// Line segment.
    pub const LINE_TO: Self = Self(0x1);

    /// Quadratic segment.
    pub const QUAD_TO: Self = Self(0x2);

    /// Cubic segment.
    pub const CUBIC_TO: Self = Self(0x3);
}

#[inline(always)]
fn estimate_number_of_quadratic_curves(c: CubicBez, accuracy: f32) -> usize {
    let q_accuracy = (accuracy * TO_QUAD_TOL) as f64;
    let max_hypot2 = 432.0 * q_accuracy * q_accuracy;
    let p1x2 = c.p1.to_vec2() * 3.0 - c.p0.to_vec2();
    let p2x2 = c.p2.to_vec2() * 3.0 - c.p3.to_vec2();
    let err = (p2x2 - p1x2).hypot2();
    let err_div = err / max_hypot2;

    estimate(err_div)
}

#[inline(always)]
fn estimate(err_div: f64) -> usize {
    // The original version of this method was:
    // let number_of_quadratic_curves = (err_div.powf(1. / 6.0).ceil() as usize).max(1);
    // number_of_quadratic_curves.min(MAX_QUADS)
    //
    // Note how we always round up and clamp to the range [1, max_quads]. Since we don't
    // care about the actual fractional value resulting from the powf call we can simply
    // compute this using a precomputed lookup table evaluating 1^6, 2^6, 3^6, etc. and simply
    // comparing if the value is less than or equal to each threshold.

    const LUT: [f64; MAX_QUADS] = [
        1.0, 64.0, 729.0, 4096.0, 15625.0, 46656.0, 117649.0, 262144.0, 531441.0, 1000000.0,
        1771561.0, 2985984.0, 4826809.0, 7529536.0, 11390625.0, 16777216.0,
    ];

    #[expect(clippy::needless_range_loop, reason = "better clarity")]
    for i in 0..MAX_QUADS {
        if err_div <= LUT[i] {
            return i + 1;
        }
    }

    MAX_QUADS
}

fn convert_cubics_to_quadratic_curves<S: Simd>(simd: S, c: &CubicBez, n: usize) {
    let number_of_quadratic_curves = n;
    let dt = 0.5 / n as f32;

    // Build each control point once as a duplicated (x,y,x,y,...) f32x8.
    // This avoids the f32x4 -> f64 reinterpret + zip path.
    #[inline(always)]
    fn splat_point_xy<S: Simd>(simd: S, x: f32, y: f32) -> f32x8<S> {
        // [x, y, x, y, x, y, x, y]
        f32x8::from_slice(simd, &[x, y, x, y, x, y, x, y])
    }

    let p0 = splat_point_xy(simd, c.p0.x as f32, c.p0.y as f32);
    let p1 = splat_point_xy(simd, c.p1.x as f32, c.p1.y as f32);
    let p2 = splat_point_xy(simd, c.p2.x as f32, c.p2.y as f32);
    let p3 = splat_point_xy(simd, c.p3.x as f32, c.p3.y as f32);

    // Cubic coefficients for Horner evaluation:
    // p(t) = ((A*t + B)*t + C)*t + D
    let coeff_a = (p1 - p2).mul_add(3.0, p3 - p0);
    let coeff_b = p1.mul_add(-2.0, p0 + p2) * 3.0;
    let coeff_c = (p1 - p0) * 3.0;
    let coeff_d = p0;

    // Evaluate 4 t-values per loop: [0,2,1,3] * dt in XY-paired lanes
    // Same lane pattern as original.
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
        if e < (number_of_quadratic_curves + 1) {
            even_pts[e] = [lo[0], lo[1]];
        }
        if e + 1 < (number_of_quadratic_curves + 1) {
            even_pts[e + 1] = [lo[2], lo[3]];
        }

        let o = i * 2;
        if o < number_of_quadratic_curves {
            odd_pts[o] = [hi[0], hi[1]];
        }
        if o + 1 < number_of_quadratic_curves {
            odd_pts[o + 1] = [hi[2], hi[3]];
        }

        t += t_inc;
    }

    even_pts[number_of_quadratic_curves] = [c.p3.x as f32, c.p3.y as f32];

    let mut out = [QuadBez::new((0.0, 0.0), (0.0, 0.0), (0.0, 0.0)); MAX_QUADS];

    for i in 0..number_of_quadratic_curves {
        let q0 = even_pts[i];
        let q2 = even_pts[i + 1];
        let ph = odd_pts[i];

        // reconstruct quadratic control point
        let q1 = [
            -0.5 * q0[0] + 2.0 * ph[0] - 0.5 * q2[0],
            -0.5 * q0[1] + 2.0 * ph[1] - 0.5 * q2[1],
        ];

        out[i] = QuadBez::new(
            (q0[0] as f64, q0[1] as f64),
            (q1[0] as f64, q1[1] as f64),
            (q2[0] as f64, q2[1] as f64),
        );
    }

    out;
}

/// Return the y inflection point or None if this curve is y-monotonic.

fn local_y_extremum_t(q: &QuadBez) -> Option<f64> {
    let div = q.p0.y - 2.0 * q.p1.y + q.p2.y;
    if div == 0.0 {
        return None;
    }

    let t = (q.p0.y - q.p1.y) / div;
    if t > 0.0 && t < 1.0 {
        Some(t)
    } else {
        None
    }
}

fn local_y_extremum_t_simd<S: Simd>(
    simd: S,
    y0: f32x4<S>,
    y1: f32x4<S>,
    y2: f32x4<S>,
) -> (f32x4<S>, u32) {
    let two = f32x4::splat(simd, 2.0);
    let zero = f32x4::splat(simd, 0.0);
    let one = f32x4::splat(simd, 1.0);

    let denom = y0 - y1 * two + y2;
    let numer = y0 - y1;
    let t = numer / denom;

    let nonzero = !denom.simd_eq(zero);
    let gt0 = t.simd_gt(zero);
    let lt1 = t.simd_lt(one);

    let valid = nonzero & gt0 & lt1;

    // mask32x4 derefs to [i32; 4]; each lane is 0xFFFFFFFF (true) or 0 (false).
    // Manually collect one bit per lane into a u32.
    let lanes: &[i32; 4] = &*valid;
    let bitmask = (lanes[0] as u32 & 1)
        | ((lanes[1] as u32 & 1) << 1)
        | ((lanes[2] as u32 & 1) << 2)
        | ((lanes[3] as u32 & 1) << 3);

    (t, bitmask)
}

fn split_quadratic_curve(q: QuadBez, _t: f32) -> (QuadBez, QuadBez) {
    let t = _t.into();
    let p01 = q.p0.lerp(q.p1, t);
    let p12 = q.p1.lerp(q.p2, t);
    let pm = p01.lerp(p12, t);
    (QuadBez::new(q.p0, p01, pm), QuadBez::new(pm, p12, q.p2))
}

fn y_monotonize_quadratic_curves<S: Simd>(quadratic_curves: &[QuadBez], simd: S) -> Vec<QuadBez> {
    quadratic_curves
        .par_chunks(4)
        .with_min_len(256)
        .flat_map(|curve| {
            let mut p0x4 = f32x4::splat(simd, 0.0);
            let mut p1x4 = f32x4::splat(simd, 0.0);
            let mut p2x4 = f32x4::splat(simd, 0.0);

            for i in 0..curve.len() {
                p0x4[i] = curve[i].p0.y as f32;
                p1x4[i] = curve[i].p1.y as f32;
                p2x4[i] = curve[i].p2.y as f32;
            }

            let (tx4, mask) = local_y_extremum_t_simd(simd, p0x4, p1x4, p2x4);
            let mut segment = Vec::with_capacity(16);

            for i in 0..curve.len() {
                if (mask & (1 << i)) != 0 {
                    let (c1, c2) = split_quadratic_curve(curve[i], tx4[i]);
                    segment.push(c1);
                    segment.push(c2);
                } else {
                    segment.push(curve[i]);
                }
            }

            segment
        })
        .collect()
}

pub struct Segment<'a> {
    tags: &'a mut Vec<PathTag>,
    segments: &'a mut Vec<u32>,
    n_segments: &'a mut u32,
    n_paths: &'a mut u32,
    first_point: [f32; 2],
    first_start_tangent_end: [f32; 2],
    state: PathState,
    n_encoded_segments: u32,
    is_fill: bool,
    transform: Affine,
}

#[derive(PartialEq)]
enum PathState {
    Start,
    MoveTo,
    NonemptySubpath,
}

impl<'a> Segment<'a> {
    pub fn move_to(&mut self, x: f32, y: f32) {
        if self.is_fill {
            self.close();
        }
        let buf = [x, y];
        let bytes = bytemuck::cast_slice(&buf);
        if self.state == PathState::MoveTo {
            let new_len = self.segments.len() - 2;
            self.segments.truncate(new_len);
        } else if self.state == PathState::NonemptySubpath {
            if !self.is_fill {
                //self.insert_stroke_cap_marker_segment(false);
            }
            if let Some(tag) = self.tags.last_mut() {
                tag.set_subpath_end();
            }
        }
        self.first_point = buf;
        self.segments.extend_from_slice(bytes);
        self.state = PathState::MoveTo;
    }

    /// Encodes a line.
    pub fn line_to(&mut self, x: f32, y: f32) {
        if self.state == PathState::Start {
            if self.n_encoded_segments == 0 {
                // This copies the behavior of kurbo which treats an initial line, quad
                // or curve as a move.
                self.move_to(x, y);
                return;
            }
            self.move_to(self.first_point[0], self.first_point[1]);
        }
        if self.state == PathState::MoveTo {
            // Ensure that we don't end up with a zero-length start tangent.
            let Some((x, y)) = self.start_tangent_for_line((x, y)) else {
                return;
            };
            self.first_start_tangent_end = [x, y];
        }
        // Drop the segment if its length is zero
        if self.is_zero_length_segment((x, y), None, None) {
            return;
        }
        let buf = [x, y];
        let bytes = bytemuck::cast_slice(&buf);
        self.segments.extend_from_slice(bytes);
        self.tags.push(PathTag::LINE_TO_F32);
        self.state = PathState::NonemptySubpath;
        self.n_encoded_segments += 1;
    }

    /// Encodes a quadratic bezier.
    pub fn quad_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        if self.state == PathState::Start {
            if self.n_encoded_segments == 0 {
                self.move_to(x2, y2);
                return;
            }
            self.move_to(self.first_point[0], self.first_point[1]);
        }
        if self.state == PathState::MoveTo {
            // Ensure that we don't end up with a zero-length start tangent.
            let Some((x, y)) = self.start_tangent_for_quad((x1, y1), (x2, y2)) else {
                return;
            };
            self.first_start_tangent_end = [x, y];
        }
        // Drop the segment if its length is zero
        if self.is_zero_length_segment((x1, y1), Some((x2, y2)), None) {
            return;
        }
        let buf = [x1, y1, x2, y2];
        let bytes = bytemuck::cast_slice(&buf);
        self.segments.extend_from_slice(bytes);
        self.tags.push(PathTag::QUAD_TO_F32);
        self.state = PathState::NonemptySubpath;
        self.n_encoded_segments += 1;
    }

    /// Encodes a cubic bezier.
    pub fn cubic_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        if self.state == PathState::Start {
            if self.n_encoded_segments == 0 {
                self.move_to(x3, y3);
                return;
            }
            self.move_to(self.first_point[0], self.first_point[1]);
        }
        if self.state == PathState::MoveTo {
            let Some((x, y)) = self.start_tangent_for_curve((x1, y1), (x2, y2), (x3, y3)) else {
                return;
            };
            self.first_start_tangent_end = [x, y];
        }
        if self.is_zero_length_segment((x1, y1), Some((x2, y2)), Some((x3, y3))) {
            return;
        }
        // note: approximate cubic curve to quadratic first!
        let buf = [x1, y1, x2, y2, x3, y3];
        let bytes = bytemuck::cast_slice(&buf);
        self.segments.extend_from_slice(bytes);
        self.tags.push(PathTag::CUBIC_TO_F32);
        self.state = PathState::NonemptySubpath;
        self.n_encoded_segments += 1;
    }

    pub fn close(&mut self) {
        match self.state {
            PathState::Start => return,
            PathState::MoveTo => {
                // If we close a new-opened path, delete it.
                let new_len = self.segments.len() - 2;
                self.segments.truncate(new_len);
                self.state = PathState::Start;
                return;
            }
            PathState::NonemptySubpath => (),
        }
        let len = self.segments.len();
        if len < 2 {
            if cfg!(debug_assertions) {
                unreachable!("There is an open path, so there must be segments.")
            }
            return;
        }
        let first_bytes = bytemuck::cast_slice(&self.first_point);
        if &self.segments[len - 2..len] != first_bytes {
            self.segments.extend_from_slice(first_bytes);
            self.tags.push(PathTag::LINE_TO_F32);
            self.n_encoded_segments += 1;
        }
        if !self.is_fill {
            // self.insert_stroke_cap_marker_segment(true);
        }
        if let Some(tag) = self.tags.last_mut() {
            tag.set_subpath_end();
        }
        self.state = PathState::Start;
    }

    fn start_tangent_for_line(&self, p1: (f32, f32)) -> Option<(f32, f32)> {
        let p0 = (self.first_point[0], self.first_point[1]);
        let pt = if (p1.0 - p0.0).abs() > EPSILON || (p1.1 - p0.1).abs() > EPSILON {
            (
                p0.0 + 1. / 3. * (p1.0 - p0.0),
                p0.1 + 1. / 3. * (p1.1 - p0.1),
            )
        } else {
            return None;
        };
        Some(pt)
    }

    fn start_tangent_for_quad(&self, p1: (f32, f32), p2: (f32, f32)) -> Option<(f32, f32)> {
        let p0 = (self.first_point[0], self.first_point[1]);
        let pt = if (p1.0 - p0.0).abs() > EPSILON || (p1.1 - p0.1).abs() > EPSILON {
            (
                p1.0 + 1. / 3. * (p0.0 - p1.0),
                p1.1 + 1. / 3. * (p0.1 - p1.1),
            )
        } else if (p2.0 - p0.0).abs() > EPSILON || (p2.1 - p0.1).abs() > EPSILON {
            (
                p1.0 + 1. / 3. * (p2.0 - p1.0),
                p1.1 + 1. / 3. * (p2.1 - p1.1),
            )
        } else {
            return None;
        };
        Some(pt)
    }

    fn start_tangent_for_curve(
        &self,
        p1: (f32, f32),
        p2: (f32, f32),
        p3: (f32, f32),
    ) -> Option<(f32, f32)> {
        let p0 = (self.first_point[0], self.first_point[1]);
        let pt = if (p1.0 - p0.0).abs() > EPSILON || (p1.1 - p0.1).abs() > EPSILON {
            p1
        } else if (p2.0 - p0.0).abs() > EPSILON || (p2.1 - p0.1).abs() > EPSILON {
            p2
        } else if (p3.0 - p0.0).abs() > EPSILON || (p3.1 - p0.1).abs() > EPSILON {
            p3
        } else {
            return None;
        };
        Some(pt)
    }

    fn is_zero_length_segment(
        &self,
        p1: (f32, f32),
        p2: Option<(f32, f32)>,
        p3: Option<(f32, f32)>,
    ) -> bool {
        let p0 = self.last_point().unwrap();
        let p2 = p2.unwrap_or(p1);
        let p3 = p3.unwrap_or(p1);

        let x_min = p0.0.min(p1.0.min(p2.0.min(p3.0)));
        let x_max = p0.0.max(p1.0.max(p2.0.max(p3.0)));
        let y_min = p0.1.min(p1.1.min(p2.1.min(p3.1)));
        let y_max = p0.1.max(p1.1.max(p2.1.max(p3.1)));

        !(x_max - x_min > EPSILON || y_max - y_min > EPSILON)
    }

    fn last_point(&self) -> Option<(f32, f32)> {
        let len = self.segments.len();
        if len < 2 {
            return None;
        }
        Some((
            bytemuck::cast(self.segments[len - 2]),
            bytemuck::cast(self.segments[len - 1]),
        ))
    }
    /// Encodes a shape.
    pub fn shape(&mut self, shape: &impl Shape) {
        self.encode_shape(shape.path_elements(0.1));
    }

    pub fn encode_shape(&mut self, path: impl Iterator<Item = peniko::kurbo::PathEl>) {
        let [a, b, c, d, tx, ty] = self.transform.as_coeffs();
        for el in path {
            match el {
                PathEl::MoveTo(p0) => self.move_to(
                    (a * p0.x + c * p0.y + tx) as f32,
                    (a * p0.x + c * p0.y + ty) as f32,
                ),
                PathEl::LineTo(p0) => self.line_to(
                    (a * p0.x + c * p0.y + tx) as f32,
                    (a * p0.x + c * p0.y + ty) as f32,
                ),
                PathEl::QuadTo(p0, p1) => {
                    self.quad_to(
                        (a * p0.x + c * p0.y + tx) as f32,
                        (a * p0.x + c * p0.y + ty) as f32,
                        (a * p1.x + c * p1.y + tx) as f32,
                        (a * p1.x + c * p1.y + tx) as f32,
                    );
                }
                PathEl::CurveTo(p0, p1, p2) => self.cubic_to(
                    (a * p0.x + c * p0.y + tx) as f32,
                    (a * p0.x + c * p0.y + ty) as f32,
                    (a * p1.x + c * p1.y + tx) as f32,
                    (a * p1.x + c * p1.y + tx) as f32,
                    (a * p2.x + c * p2.y + tx) as f32,
                    (a * p2.x + c * p2.y + tx) as f32,
                ),
                PathEl::ClosePath => self.close(),
            }
        }
    }
}
