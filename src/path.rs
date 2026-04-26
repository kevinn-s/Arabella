use bytemuck::{Pod, Zeroable};
use fearless_simd::*;
use kurbo::{
    Affine, CubicBez, ParamCurve, PathEl, Point, QuadBez, Shape,
};
use alloc::vec;
use alloc::vec::Vec;
const MAX_QUADS: usize = 16;
const TO_QUAD_TOL: f32 = 16.0;
const EPSILON: f32 = 1e-12;
pub(crate) const SQRT_TOL: f64 = 0.5;
pub(crate) const TOL: f64 = SQRT_TOL * SQRT_TOL;

// ============================================================================
// PathTag (kept for compatibility / reference, but no longer used internally)
// ============================================================================

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct PathTag(pub u8);

impl PathTag {
    pub const LINE_TO_F32: Self = Self(0x9);
    pub const QUAD_TO_F32: Self = Self(0xa);
    pub const CUBIC_TO_F32: Self = Self(0xb);

    pub const TRANSFORM: Self = Self(0x20);
    pub const PATH: Self = Self(0x10);
    pub const STYLE: Self = Self(0x40);

    pub const SUBPATH_END_BIT: u8 = 0x4;
    const F32_BIT: u8 = 0x8;
    const SEGMENT_MASK: u8 = 0x3;

    pub fn is_path_segment(self) -> bool {
        self.path_segment_type().0 != 0
    }

    pub fn is_subpath_end(self) -> bool {
        self.0 & Self::SUBPATH_END_BIT != 0
    }

    pub fn set_subpath_end(&mut self) {
        self.0 |= Self::SUBPATH_END_BIT;
    }

    pub fn path_segment_type(self) -> PathSegmentType {
        PathSegmentType(self.0 & Self::SEGMENT_MASK)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Pod, Zeroable)]
#[repr(C)]
pub struct PathSegmentType(pub u8);

impl PathSegmentType {
    pub const LINE_TO: Self = Self(0x1);
    pub const QUAD_TO: Self = Self(0x2);
    pub const CUBIC_TO: Self = Self(0x3);
}

// ============================================================================
// Cubic-to-quadratic conversion (unchanged)
// ============================================================================

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
    c: &CubicBez,
    n: usize,
) -> [f32; 96] {
    let dt = 0.5 / n as f32;

    #[inline(always)]
    fn splat_point_xy<S: Simd>(simd: S, x: f32, y: f32) -> f32x8<S> {
        f32x8::from_slice(simd, &[x, y, x, y, x, y, x, y])
    }

    let p0 = splat_point_xy(simd, c.p0.x as f32, c.p0.y as f32);
    let p1 = splat_point_xy(simd, c.p1.x as f32, c.p1.y as f32);
    let p2 = splat_point_xy(simd, c.p2.x as f32, c.p2.y as f32);
    let p3 = splat_point_xy(simd, c.p3.x as f32, c.p3.y as f32);

    let coeff_a = (p1 - p2).mul_add(3.0, p3 - p0);
    let coeff_b = p1.mul_add(-2.0, p0 + p2) * 3.0;
    let coeff_c = (p1 - p0) * 3.0;
    let coeff_d = p0;

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

    even_pts[n] = [c.p3.x as f32, c.p3.y as f32];

    let mut out: [f32; 96] = [0.0; 96];
    let mut index = 0;
    for i in 0..n.div_ceil(4) {
        let p0v = f32x8::from_slice(simd, &even_pts.as_flattened()[i * 8..][..8]);
        let p_onehalf = f32x8::from_slice(simd, &odd_pts.as_flattened()[i * 8..][..8]);
        let p2v = f32x8::from_slice(simd, &even_pts.as_flattened()[(i * 8 + 2)..][..8]);
        let x = p0v * -0.5;
        let x1 = p_onehalf.mul_add(2.0, x);
        let p1v = p2v.mul_add(-0.5, x1);
        let p0_slice = p0v.as_slice();
        let p1_slice = p1v.as_slice();
        for j in 0..4 {
            out[index]     = p0_slice[j * 2];
            out[index + 1] = p0_slice[j * 2 + 1];
            out[index + 2] = p1_slice[j * 2];
            out[index + 3] = p1_slice[j * 2 + 1];
            index += 4;
        }
    }

    let p2v = f32x8::from_slice(
        simd,
        &even_pts.as_flattened()[(n.div_ceil(4) * 8 + 2)..][..8],
    );
    let p2_slice = p2v.as_slice();
    out[index]     = p2_slice[0];
    out[index + 1] = p2_slice[1];

    out
}

// ============================================================================
// Y-monotonization helpers
// ============================================================================

/// Returns the y-extremum t-value if the quadratic is not y-monotonic.
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

fn split_quadratic_curve(q: QuadBez, t: f32) -> (QuadBez, QuadBez) {
    let t = t as f64;
    let p01 = q.p0.lerp(q.p1, t);
    let p12 = q.p1.lerp(q.p2, t);
    let pm = p01.lerp(p12, t);
    (
        QuadBez::new(q.p0, p01, pm),
        QuadBez::new(pm, p12, q.p2),
    )
}

// ============================================================================
// PathEncoder
// ============================================================================

#[derive(PartialEq)]
enum PathState {
    Start,
    MoveTo,
    NonemptySubpath,
}

/// Encodes path commands into SoA storage:
/// - `lines`: stride 4 floats (p0x, p0y, p1x, p1y), stored as u32 bit patterns
/// - `quads`: stride 6 floats (p0x, p0y, p1x, p1y, p2x, p2y), stored as u32 bit patterns
///
/// Quadratics are y-monotonized inline during encoding.
/// Cubics are subdivided into multiple quadratics (each then y-monotonized).
pub struct PathEncoder<'a, S: Simd> {
    lines: &'a mut Vec<u32>,
    quads: &'a mut Vec<u32>,
    n_segments: &'a mut u32,
    n_paths: &'a mut u32,
    first_point: [f32; 2],
    current_point: [f32; 2],
    state: PathState,
    n_encoded_segments: u32,
    is_fill: bool,
    transform: Affine,
    simd: S,
}

impl<'a, S: Simd> PathEncoder<'a, S> {
    pub fn new(
        lines: &'a mut Vec<u32>,
        quads: &'a mut Vec<u32>,
        n_segments: &'a mut u32,
        n_paths: &'a mut u32,
        is_fill: bool,
        transform: Affine,
        simd: S,
    ) -> Self {
        Self {
            lines,
            quads,
            n_segments,
            n_paths,
            first_point: [0.0, 0.0],
            current_point: [0.0, 0.0],
            state: PathState::Start,
            n_encoded_segments: 0,
            is_fill,
            transform,
            simd,
        }
    }

    // ---- Path command encoding ----

    pub fn move_to(&mut self, x: f32, y: f32) {
        if self.is_fill {
            self.close();
        }
        // For fills, we don't emit a separate move-to — we just remember the starting point.
        // Subsequent line/quad commands will use current_point as their p0.
        self.first_point = [x, y];
        self.current_point = [x, y];
        self.state = PathState::MoveTo;
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        // Treat an initial line as a move (kurbo behavior compatibility).
        if self.state == PathState::Start {
            if self.n_encoded_segments == 0 {
                self.move_to(x, y);
                return;
            }
            self.move_to(self.first_point[0], self.first_point[1]);
        }

        let p0x = self.current_point[0];
        let p0y = self.current_point[1];

        // Drop zero-length segments
        if (x - p0x).abs() <= EPSILON && (y - p0y).abs() <= EPSILON {
            return;
        }

        self.lines.push(p0x.to_bits());
        self.lines.push(p0y.to_bits());
        self.lines.push(x.to_bits());
        self.lines.push(y.to_bits());

        self.current_point = [x, y];
        self.state = PathState::NonemptySubpath;
        self.n_encoded_segments += 1;
    }

    pub fn quad_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        if self.state == PathState::Start {
            if self.n_encoded_segments == 0 {
                self.move_to(x2, y2);
                return;
            }
            self.move_to(self.first_point[0], self.first_point[1]);
        }

        let p0x = self.current_point[0];
        let p0y = self.current_point[1];

        // Drop zero-length quads (bbox collapsed to a point)
        let xmin = p0x.min(x1).min(x2);
        let xmax = p0x.max(x1).max(x2);
        let ymin = p0y.min(y1).min(y2);
        let ymax = p0y.max(y1).max(y2);
        if (xmax - xmin) <= EPSILON && (ymax - ymin) <= EPSILON {
            return;
        }

        let q = QuadBez::new(
            Point::new(p0x as f64, p0y as f64),
            Point::new(x1 as f64, y1 as f64),
            Point::new(x2 as f64, y2 as f64),
        );

        // Y-monotonize inline: split at y-extremum if any
        if let Some(t) = local_y_extremum_t(&q) {
            let (a, b) = split_quadratic_curve(q, t as f32);
            self.push_quad(a);
            self.push_quad(b);
        } else {
            self.push_quad(q);
        }

        self.current_point = [x2, y2];
        self.state = PathState::NonemptySubpath;
        self.n_encoded_segments += 1;
    }

    pub fn cubic_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        if self.state == PathState::Start {
            if self.n_encoded_segments == 0 {
                self.move_to(x3, y3);
                return;
            }
            self.move_to(self.first_point[0], self.first_point[1]);
        }

        let p0x = self.current_point[0];
        let p0y = self.current_point[1];

        // Drop zero-length cubics
        let xmin = p0x.min(x1).min(x2).min(x3);
        let xmax = p0x.max(x1).max(x2).max(x3);
        let ymin = p0y.min(y1).min(y2).min(y3);
        let ymax = p0y.max(y1).max(y2).max(y3);
        if (xmax - xmin) <= EPSILON && (ymax - ymin) <= EPSILON {
            return;
        }

        let c = CubicBez::new(
            Point::new(p0x as f64, p0y as f64),
            Point::new(x1 as f64, y1 as f64),
            Point::new(x2 as f64, y2 as f64),
            Point::new(x3 as f64, y3 as f64),
        );

        let n = estimate_number_of_quadratic_curves(c, TOL as f32);
        let q_buf = convert_cubics_to_quadratic_curves(self.simd, &c, n);

        // The output format from `convert_cubics_to_quadratic_curves` is:
        //   [P0_0.x, P0_0.y, P1_0.x, P1_0.y,
        //    P0_1.x, P0_1.y, P1_1.x, P1_1.y,
        //    ...,
        //    P2_final.x, P2_final.y]
        //
        // Each sub-quad i has control points (P0_i, P1_i, P2_i) where
        // P2_i == P0_(i+1) (shared endpoint), and the final P2 sits at the end.
        //
        // We feed each sub-quad through quad_to() so y-monotonization happens
        // automatically per sub-quad. quad_to() reads p0 from current_point.

        for i in 0..n {
            let base = i * 4;
            // sub-quad i: P0 = current_point (already correct), P1 = q_buf[base+2..4],
            // P2 = q_buf[base+4..6] (which is the next P0, or final P2 if i == n-1)
            let p1x = q_buf[base + 2];
            let p1y = q_buf[base + 3];
            let p2x = q_buf[base + 4];
            let p2y = q_buf[base + 5];
            self.quad_to(p1x, p1y, p2x, p2y);
        }

        // current_point is now (x3, y3) via the last quad_to call
    }

    pub fn close(&mut self) {
        match self.state {
            PathState::Start => return,
            PathState::MoveTo => {
                // Closing an empty subpath — nothing to do
                self.state = PathState::Start;
                return;
            }
            PathState::NonemptySubpath => (),
        }

        // If current point != first point, emit a closing line
        let cx = self.current_point[0];
        let cy = self.current_point[1];
        let fx = self.first_point[0];
        let fy = self.first_point[1];
        if (cx - fx).abs() > EPSILON || (cy - fy).abs() > EPSILON {
            // Emit a line from current to first
            self.lines.push(cx.to_bits());
            self.lines.push(cy.to_bits());
            self.lines.push(fx.to_bits());
            self.lines.push(fy.to_bits());
            self.n_encoded_segments += 1;
        }

        self.current_point = self.first_point;
        self.state = PathState::Start;
    }

    /// Encodes a kurbo Shape.
    pub fn shape(&mut self, shape: &impl Shape) {
        self.encode_shape(shape.path_elements(0.1));
    }

    /// Encodes a sequence of path elements with the active affine transform applied.
    pub fn encode_shape(&mut self, path: impl Iterator<Item = PathEl>) {
        let [a, b, c, d, tx, ty] = self.transform.as_coeffs();
        let tx_f = |x: f64, y: f64| (a * x + c * y + tx) as f32;
        let ty_f = |x: f64, y: f64| (b * x + d * y + ty) as f32;

        for el in path {
            match el {
                PathEl::MoveTo(p) => {
                    self.move_to(tx_f(p.x, p.y), ty_f(p.x, p.y));
                }
                PathEl::LineTo(p) => {
                    self.line_to(tx_f(p.x, p.y), ty_f(p.x, p.y));
                }
                PathEl::QuadTo(p1, p2) => {
                    self.quad_to(
                        tx_f(p1.x, p1.y), ty_f(p1.x, p1.y),
                        tx_f(p2.x, p2.y), ty_f(p2.x, p2.y),
                    );
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    self.cubic_to(
                        tx_f(p1.x, p1.y), ty_f(p1.x, p1.y),
                        tx_f(p2.x, p2.y), ty_f(p2.x, p2.y),
                        tx_f(p3.x, p3.y), ty_f(p3.x, p3.y),
                    );
                }
                PathEl::ClosePath => self.close(),
            }
        }

        // Auto-close any open subpath at the end (for fills)
        if self.is_fill && self.state == PathState::NonemptySubpath {
            self.close();
        }

        *self.n_paths += 1;
        *self.n_segments += self.n_encoded_segments;
    }

    // ---- Internal helpers ----

    /// Push a y-monotonic quadratic into the SoA quads buffer.
    fn push_quad(&mut self, q: QuadBez) {
        self.quads.push((q.p0.x as f32).to_bits());
        self.quads.push((q.p0.y as f32).to_bits());
        self.quads.push((q.p1.x as f32).to_bits());
        self.quads.push((q.p1.y as f32).to_bits());
        self.quads.push((q.p2.x as f32).to_bits());
        self.quads.push((q.p2.y as f32).to_bits());
    }
}