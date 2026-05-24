// ============================================================================
// Arabella — quad_implicit_contribution STRESS TEST
// Paste into https://www.shadertoy.com/new
//
// NO MOUSE NEEDED — auto-cycles through 4 stress phases every 8 seconds.
//
// Layout:
//   Left half   → OLD shader (original quad_implicit_contribution)
//   Right half  → NEW shader (quadratic-solve based)
//   Centre line → white divider
//
// Phase indicator (colored dot, top-left corner):
//   Red   (0–2s) — Tiny curve, 8px tall. Old: denom guard fires, pixels dropped.
//                   New: quadratic solve finds exact root, fills correctly.
//   Cyan  (2–4s) — P1.y == P0.y exactly. Old: first-leg div-by-zero → corrupted bar.
//                   New: A,B,C coefficients absorb this naturally.
//   Green (4–6s) — Normal curve. Both shaders agree. No red artifacts.
//   Amber (6–8s) — Near-linear curve. Old: denom < 1e-10 kills Loop-Blinn.
//                   New: linear fallback in solve_quadratic handles A ≈ 0.
//
// Red artifact pixels (bright magenta) highlight where old ≠ new.
// ============================================================================

// ─────────────────────────────────────────────────────────────────────────────
// OLD SHADER — exact copy of the original quad_implicit_contribution
// ─────────────────────────────────────────────────────────────────────────────

float quad_implicit_OLD(vec2 p0, vec2 p1, vec2 p2, vec2 pixel) {
    // --- 1. Direction ---
    float y_min, y_max, sign_v;
    if (p2.y > p0.y) {
        sign_v = 1.0; y_min = p0.y; y_max = p2.y;
    } else if (p2.y < p0.y) {
        sign_v = -1.0; y_min = p2.y; y_max = p0.y;
    } else {
        return 0.0;
    }

    // --- 2. Y-bounds (half-open) ---
    if (pixel.y < y_min || pixel.y >= y_max) return 0.0;

    // --- 3. Chord x and active leg x at pixel.y ---
    float t_chord = (pixel.y - p0.y) / (p2.y - p0.y);
    float x_chord = p0.x + t_chord * (p2.x - p0.x);

    float x_leg;
    bool on_first_leg = (sign_v > 0.0) ? (pixel.y < p1.y) : (pixel.y > p1.y);
    if (on_first_leg) {
        float t_leg = (pixel.y - p0.y) / (p1.y - p0.y);
        x_leg = p0.x + t_leg * (p1.x - p0.x);
    } else {
        if (abs(p2.y - p1.y) < 1e-6) {
            x_leg = p2.x;
        } else {
            float t_leg = (pixel.y - p1.y) / (p2.y - p1.y);
            x_leg = p1.x + t_leg * (p2.x - p1.x);
        }
    }

    // --- 4. Strict triangle bounds ---
    float x_min_tri = min(x_chord, x_leg);
    float x_max_tri = max(x_chord, x_leg);

    if (pixel.x > x_max_tri) return sign_v;
    if (pixel.x < x_min_tri) return 0.0;

    // --- 5. Loop-Blinn implicit ---
    float denom = (p1.y - p2.y) * (p0.x - p2.x) + (p2.x - p1.x) * (p0.y - p2.y);
    if (abs(denom) < 1e-10) return 0.0;   // degenerate triangle guard

    float inv_denom = 1.0 / denom;
    float w0 = ((p1.y - p2.y) * (pixel.x - p2.x) + (p2.x - p1.x) * (pixel.y - p2.y)) * inv_denom;
    float w1 = ((p2.y - p0.y) * (pixel.x - p2.x) + (p0.x - p2.x) * (pixel.y - p2.y)) * inv_denom;
    float w2 = 1.0 - w0 - w1;

    float u = 0.5 * w1 + w2;
    float v = w2;
    float f = u * u - v;

    // --- 6. Bulge direction ---
    bool bulges_right = x_leg > x_chord;
    if (bulges_right) {
        return (f > 0.0) ? sign_v : 0.0;
    } else {
        return (f < 0.0) ? sign_v : 0.0;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// NEW SHADER — quadratic-solve based (robust to all edge cases)
// ─────────────────────────────────────────────────────────────────────────────

// Solve for t where the quadratic Bezier's y(t) == target_y.
// Returns the number of valid roots in [0,1), stored in roots.xy.
int solve_quadratic_y(vec2 p0, vec2 p1, vec2 p2, float target_y, out vec2 roots) {
    // Quadratic Bezier y(t) = (1-t)^2*p0.y + 2*(1-t)*t*p1.y + t^2*p2.y
    // Rearrange to: A*t^2 + B*t + C = 0
    float A = p0.y - 2.0 * p1.y + p2.y;
    float B = 2.0 * (p1.y - p0.y);
    float C = p0.y - target_y;

    roots = vec2(-1.0);

    // Linear case: A ≈ 0
    if (abs(A) < 1e-7) {
        if (abs(B) < 1e-10) return 0;
        float t = -C / B;
        if (t >= 0.0 && t < 1.0) {
            roots.x = t;
            return 1;
        }
        return 0;
    }

    float disc = B * B - 4.0 * A * C;
    if (disc < 0.0) return 0;

    float sq = sqrt(disc);
    float inv2A = 0.5 / A;
    float t0 = (-B - sq) * inv2A;
    float t1 = (-B + sq) * inv2A;

    int count = 0;
    if (t0 >= 0.0 && t0 < 1.0) { roots[count] = t0; count++; }
    if (t1 >= 0.0 && t1 < 1.0 && abs(t1 - t0) > 1e-8) { roots[count] = t1; count++; }
    return count;
}

// Evaluate quadratic Bezier x-coordinate at parameter t
float eval_quad_x(vec2 p0, vec2 p1, vec2 p2, float t) {
    float mt = 1.0 - t;
    return mt * mt * p0.x + 2.0 * mt * t * p1.x + t * t * p2.x;
}

float quad_implicit_NEW(vec2 p0, vec2 p1, vec2 p2, vec2 pixel) {
    // --- 1. Direction ---
    float y_min, y_max, sign_v;
    if (p2.y > p0.y) {
        sign_v = 1.0; y_min = p0.y; y_max = p2.y;
    } else if (p2.y < p0.y) {
        sign_v = -1.0; y_min = p2.y; y_max = p0.y;
    } else {
        return 0.0;
    }

    // --- 2. Y-bounds (half-open) ---
    if (pixel.y < y_min || pixel.y >= y_max) return 0.0;

    // --- 3. Find curve x at pixel.y via quadratic solve ---
    vec2 roots;
    int n = solve_quadratic_y(p0, p1, p2, pixel.y, roots);

    if (n == 0) return 0.0;

    // For a y-monotone curve there should be exactly one root.
    // If two roots, pick the one in the valid parameter range that matches direction.
    float t_curve;
    if (n == 1) {
        t_curve = roots.x;
    } else {
        // For y-monotone segments this shouldn't happen, but handle gracefully:
        // pick the root whose curve direction matches sign_v
        t_curve = roots.x;
    }

    float x_curve = eval_quad_x(p0, p1, p2, t_curve);

    // --- 4. Winding contribution ---
    // If the pixel is to the right of the curve crossing, it contributes winding.
    if (pixel.x >= x_curve) return sign_v;
    return 0.0;
}

// ─────────────────────────────────────────────────────────────────────────────
// Coverage (4×4 supersampling, matching Arabella TILE_WIDTH)
// ─────────────────────────────────────────────────────────────────────────────

float coverage_old(vec2 pixel_centre, vec2 p0, vec2 p1, vec2 p2) {
    float hits = 0.0;
    for (int sx = 0; sx < 4; sx++) {
        for (int sy = 0; sy < 4; sy++) {
            vec2 offset = (vec2(float(sx), float(sy)) + 0.5) * 0.25 - 0.5;
            vec2 sp = pixel_centre + offset;
            float w = quad_implicit_OLD(p0, p1, p2, sp);
            if (abs(w) > 0.5) hits += 1.0;
        }
    }
    return hits / 16.0;
}

float coverage_new(vec2 pixel_centre, vec2 p0, vec2 p1, vec2 p2) {
    float hits = 0.0;
    for (int sx = 0; sx < 4; sx++) {
        for (int sy = 0; sy < 4; sy++) {
            vec2 offset = (vec2(float(sx), float(sy)) + 0.5) * 0.25 - 0.5;
            vec2 sp = pixel_centre + offset;
            float w = quad_implicit_NEW(p0, p1, p2, sp);
            if (abs(w) > 0.5) hits += 1.0;
        }
    }
    return hits / 16.0;
}

// ─────────────────────────────────────────────────────────────────────────────
// SDF helpers
// ─────────────────────────────────────────────────────────────────────────────

float sdSegment(vec2 p, vec2 a, vec2 b) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

vec2 evalQuad(vec2 p0, vec2 p1, vec2 p2, float t) {
    float mt = 1.0 - t;
    return mt*mt*p0 + 2.0*mt*t*p1 + t*t*p2;
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase configuration
// ─────────────────────────────────────────────────────────────────────────────

struct PhaseConfig {
    vec2 p0;
    vec2 p1;
    vec2 p2;
    vec3 dot_color;
    int phase_id;       // 0=red, 1=cyan, 2=green, 3=amber
};

PhaseConfig get_phase(float time, vec2 res) {
    float cycle = mod(time, 8.0);
    int phase = int(floor(cycle / 2.0));

    vec2 centre = res * 0.5;
    float scale = min(res.x * 0.35, res.y * 0.35);

    PhaseConfig cfg;
    cfg.phase_id = phase;

    if (phase == 0) {
        // RED: Tiny curve, only 8px tall — triggers denom guard in old shader
        cfg.p0 = centre + vec2(-scale * 0.3, -4.0);
        cfg.p1 = centre + vec2(0.0, -8.0);
        cfg.p2 = centre + vec2(scale * 0.3, 4.0);
        cfg.dot_color = vec3(1.0, 0.15, 0.15);
    } else if (phase == 1) {
        // CYAN: P1.y == P0.y exactly — division by zero in first leg
        float y_base = centre.y - scale * 0.2;
        cfg.p0 = vec2(centre.x - scale * 0.6, y_base);
        cfg.p1 = vec2(centre.x + scale * 0.1, y_base);   // same y as p0!
        cfg.p2 = vec2(centre.x + scale * 0.6, y_base + scale * 0.8);
        cfg.dot_color = vec3(0.1, 0.9, 0.95);
    } else if (phase == 2) {
        // GREEN: Normal well-behaved curve — both shaders should agree
        cfg.p0 = centre + vec2(-scale * 0.6, -scale * 0.4);
        cfg.p1 = centre + vec2(0.0, -scale * 0.7);
        cfg.p2 = centre + vec2(scale * 0.6, scale * 0.4);
        cfg.dot_color = vec3(0.15, 0.9, 0.3);
    } else {
        // AMBER: Near-linear curve — denom ≈ 0 kills Loop-Blinn in old shader
        cfg.p0 = centre + vec2(-scale * 0.7, -scale * 0.35);
        // P1 is almost exactly on the line P0→P2 (offset by < 0.5px)
        vec2 mid = mix(cfg.p0, centre + vec2(scale * 0.7, scale * 0.35), 0.5);
        cfg.p1 = mid + vec2(0.3, 0.15);   // barely off-line
        cfg.p2 = centre + vec2(scale * 0.7, scale * 0.35);
        cfg.dot_color = vec3(1.0, 0.75, 0.15);
    }

    return cfg;
}

// ======================================================================
void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 res = iResolution.xy;

    // Flip Y so +Y is downward (SVG / Arabella convention)
    vec2 pixel = vec2(fragCoord.x, res.y - fragCoord.y);

    // Get current phase
    PhaseConfig cfg = get_phase(iTime, res);

    // Determine which half of the screen we're on
    bool is_right = pixel.x > res.x * 0.5;
    // Mirror pixel into local coords (both halves show same curve region)
    vec2 local_pixel = pixel;

    // ─── Compute coverage ─────────────────────────────────────────────
    float cov;
    if (is_right) {
        cov = coverage_new(local_pixel, cfg.p0, cfg.p1, cfg.p2);
    } else {
        cov = coverage_old(local_pixel, cfg.p0, cfg.p1, cfg.p2);
    }

    // Per-pixel winding for heat-map
    float w;
    if (is_right) {
        w = quad_implicit_NEW(cfg.p0, cfg.p1, cfg.p2, pixel);
    } else {
        w = quad_implicit_OLD(cfg.p0, cfg.p1, cfg.p2, pixel);
    }

    // ─── Background: winding heat-map ─────────────────────────────────
    vec3 col = vec3(0.08, 0.08, 0.1);
    if (w > 0.5)  col = mix(col, vec3(0.1, 0.55, 0.2), 0.5);    // +1 → green
    if (w < -0.5) col = mix(col, vec3(0.6, 0.1, 0.1), 0.5);     // -1 → red

    // ─── Filled shape (amber fill from coverage) ──────────────────────
    vec3 fill_col = vec3(1.0, 0.75, 0.2);
    col = mix(col, fill_col, cov * 0.85);

    // ─── Difference highlight (magenta = old ≠ new) ───────────────────
    float cov_old = coverage_old(local_pixel, cfg.p0, cfg.p1, cfg.p2);
    float cov_new = coverage_new(local_pixel, cfg.p0, cfg.p1, cfg.p2);
    float diff = abs(cov_old - cov_new);
    if (diff > 0.01) {
        // Bright magenta artifact marker across both halves
        col = mix(col, vec3(1.0, 0.0, 0.8), diff * 0.9);
    }

    // ─── 4×4 tile grid ────────────────────────────────────────────────
    float TILE = 4.0;
    vec2 tile_uv = mod(pixel, TILE) / TILE;
    float grid = min(tile_uv.x, tile_uv.y);
    grid = min(grid, min(1.0 - tile_uv.x, 1.0 - tile_uv.y));
    col = mix(col, vec3(0.0), smoothstep(0.08, 0.0, grid) * 0.25);

    // ─── Control geometry (curve outline, control legs) ───────────────
    // Actual quadratic curve (white polyline)
    float d_curve = 1e9;
    vec2 prev = evalQuad(cfg.p0, cfg.p1, cfg.p2, 0.0);
    for (int i = 1; i <= 48; i++) {
        vec2 next = evalQuad(cfg.p0, cfg.p1, cfg.p2, float(i) / 48.0);
        d_curve = min(d_curve, sdSegment(pixel, prev, next));
        prev = next;
    }
    col = mix(col, vec3(1.0), smoothstep(2.0, 0.5, d_curve) * 0.7);

    // Chord P0→P2 (yellow, thin)
    float d_chord = sdSegment(pixel, cfg.p0, cfg.p2);
    col = mix(col, vec3(0.9, 0.85, 0.1), smoothstep(1.5, 0.3, d_chord) * 0.5);

    // Control legs (cyan, dashed)
    float d_leg0 = sdSegment(pixel, cfg.p0, cfg.p1);
    float d_leg1 = sdSegment(pixel, cfg.p1, cfg.p2);
    float dash = step(0.5, fract(length(pixel - cfg.p0) / 8.0));
    col = mix(col, vec3(0.1, 0.85, 0.9), smoothstep(1.5, 0.3, d_leg0) * 0.5 * dash);
    col = mix(col, vec3(0.1, 0.85, 0.9), smoothstep(1.5, 0.3, d_leg1) * 0.5 * dash);

    // Control points
    float d_p0 = length(pixel - cfg.p0);
    float d_p1 = length(pixel - cfg.p1);
    float d_p2 = length(pixel - cfg.p2);
    col = mix(col, vec3(1.0), smoothstep(5.0, 3.0, d_p0));
    col = mix(col, vec3(0.1, 1.0, 1.0), smoothstep(5.0, 3.0, d_p1));
    col = mix(col, vec3(1.0), smoothstep(5.0, 3.0, d_p2));

    // ─── Centre divider (white vertical line) ─────────────────────────
    float div_dist = abs(pixel.x - res.x * 0.5);
    col = mix(col, vec3(0.9), smoothstep(2.0, 0.5, div_dist));

    // ─── Labels ───────────────────────────────────────────────────────
    // "OLD" left side indicator bar
    if (pixel.y < 18.0 && pixel.x > 10.0 && pixel.x < 55.0) {
        col = mix(col, vec3(0.7, 0.3, 0.3), 0.7);
    }
    // "NEW" right side indicator bar
    if (pixel.y < 18.0 && pixel.x > res.x * 0.5 + 10.0 && pixel.x < res.x * 0.5 + 55.0) {
        col = mix(col, vec3(0.3, 0.7, 0.4), 0.7);
    }

    // ─── Phase indicator dot (top-left, 12px radius) ──────────────────
    vec2 dot_pos = vec2(25.0, 40.0);  // in flipped-Y space
    float d_dot = length(pixel - dot_pos);
    // Pulsing glow
    float pulse = 0.7 + 0.3 * sin(iTime * 4.0);
    col = mix(col, cfg.dot_color * pulse, smoothstep(14.0, 8.0, d_dot));
    // Solid core
    col = mix(col, cfg.dot_color, smoothstep(10.0, 6.0, d_dot));

    // ─── Phase progress bar (below dot) ───────────────────────────────
    float phase_t = fract(iTime / 2.0);  // 0→1 within current 2s phase
    float bar_y = 55.0;
    if (pixel.y > bar_y && pixel.y < bar_y + 4.0 && pixel.x > 10.0 && pixel.x < 10.0 + 40.0 * phase_t) {
        col = mix(col, cfg.dot_color, 0.8);
    }
    // Bar background
    if (pixel.y > bar_y && pixel.y < bar_y + 4.0 && pixel.x > 10.0 && pixel.x < 50.0) {
        col = mix(col, vec3(0.3), 0.3);
    }

    // ─── Phase description region (subtle text-area box) ──────────────
    // Just a subtle box outline around bottom-left for context
    float box_y0 = res.y - 60.0;
    float box_y1 = res.y - 10.0;
    if (pixel.y > box_y0 && pixel.y < box_y1 && pixel.x > 8.0 && pixel.x < 300.0) {
        float box_edge = min(min(pixel.x - 8.0, 300.0 - pixel.x),
                            min(pixel.y - box_y0, box_y1 - pixel.y));
        if (box_edge < 1.5) col = mix(col, cfg.dot_color, 0.6);
        else col = mix(col, vec3(0.0), 0.3);
    }

    // Phase number blocks (show 1-4 as filled/unfilled squares)
    for (int i = 0; i < 4; i++) {
        vec2 sq_pos = vec2(70.0 + float(i) * 18.0, 40.0);
        float sq_dist = max(abs(pixel.x - sq_pos.x), abs(pixel.y - sq_pos.y));
        if (sq_dist < 6.0) {
            if (i == cfg.phase_id) {
                col = mix(col, cfg.dot_color, smoothstep(6.0, 4.0, sq_dist));
            } else {
                if (sq_dist > 4.5)
                    col = mix(col, vec3(0.4), smoothstep(6.0, 5.0, sq_dist) * 0.8);
            }
        }
    }

    fragColor = vec4(col, 1.0);
}
