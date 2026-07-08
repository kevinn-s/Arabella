use alloc::{format, vec::Vec};

/// Flattens a quadratic Bézier curve into line segments using Blaze's
/// recursive midpoint subdivision method.
///
/// All coordinates are in 24.8 fixed point (i32 where 256 = 1 pixel).
///
/// # Arguments
/// * `p0x, p0y` — start point (24.8)
/// * `p1x, p1y` — control point (24.8)
/// * `p2x, p2y` — end point (24.8)
/// * `callback`  — called once per output line segment with (x0, y0, x1, y1) in 24.8
///
/// # Flatness tolerance
/// Uses L1 distance from chord midpoint to control point.
/// Threshold of 256 = 1.0 pixel L1 deviation (~0.5 px max curve deviation).
/// For coarser binning, increase to 512 or 768.
const FLATNESS_THRESHOLD: i32 = 32;

pub fn flatten_quadratic<F>(
    p0x: i32, p0y: i32,
    p1x: i32, p1y: i32,
    p2x: i32, p2y: i32,
    callback: &mut F,
) where
    F: FnMut(i32, i32, i32, i32),
{
    flatten_recursive(p0x, p0y, p1x, p1y, p2x, p2y, callback);
}

fn flatten_recursive<F>(
    p0x: i32, p0y: i32,
    p1x: i32, p1y: i32,
    p2x: i32, p2y: i32,
    callback: &mut F,
) where
    F: FnMut(i32, i32, i32, i32),
{
    if is_flat_enough(p0x, p0y, p1x, p1y, p2x, p2y) {
        callback(p0x, p0y, p2x, p2y);

    } else {
        // Not flat — split at midpoint (De Casteljau).
        let m01x = (p0x + p1x) / 2;
        let m01y = (p0y + p1y) / 2;
        let m12x = (p1x + p2x) / 2;
        let m12y = (p1y + p2y) / 2;
        let midx = (m01x + m12x) / 2;
        let midy = (m01y + m12y) / 2;

        flatten_recursive(p0x, p0y, m01x, m01y, midx, midy, callback);
        flatten_recursive(midx, midy, m12x, m12y, p2x, p2y, callback);
    }
}

#[inline(always)]
fn is_flat_enough(
    p0x: i32, p0y: i32,
    p1x: i32, p1y: i32,
    p2x: i32, p2y: i32,
) -> bool {
    // Degenerate case: start == end.
    if p0x == p2x && p0y == p2y {
        return true;
    }

    // Midpoint of chord p0→p2:
    let mx = (p0x + p2x) / 2;
    let my = (p0y + p2y) / 2;

    // L1 distance from midpoint to control point:
    let dx = (mx - p1x).abs();
    let dy = (my - p1y).abs();
    let dc = dx + dy;

    dc <= FLATNESS_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_already_flat() {
        // Straight line: control point on the chord.
        let mut lines = Vec::new();
        flatten_quadratic(0, 0, 128, 128, 256, 256, &mut |x0, y0, x1, y1| {
            lines.push((x0, y0, x1, y1));
        });
        // Should produce exactly 1 line.
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], (0, 0, 256, 256));
    }

    #[test]
    fn test_needs_split() {
        // Curve with control point far from chord — should split.
        let mut lines = Vec::new();
        flatten_quadratic(0, 0, 512, 1024, 1024, 0, &mut |x0, y0, x1, y1| {
            lines.push((x0, y0, x1, y1));
        });
        // Should produce more than 1 line.
        assert!(lines.len() > 1);
        // First line starts at p0:
        assert_eq!(lines[0].0, 0);
        assert_eq!(lines[0].1, 0);
        // Last line ends at p2:
        let last = lines.last().unwrap();
        assert_eq!(last.2, 1024);
        assert_eq!(last.3, 0);
        // Continuity: each line's end = next line's start.
        for i in 0..lines.len() - 1 {
            assert_eq!(lines[i].2, lines[i + 1].0);
            assert_eq!(lines[i].3, lines[i + 1].1);
        }
    }
}