// All icons are pure egui Painter primitives — keeps the binary small and
// guarantees pixel-identical rendering across platforms. Sized in pixels;
// strokes are clamped to a minimum of ~1.5px so they stay crisp at small sizes.
//
// Some icons are part of the design system surface and may not be referenced
// from any call site yet; allow dead code so the catalog stays complete.
#![allow(dead_code)]

use egui::{vec2, Color32, Painter, Pos2, Rect, Rounding, Shape, Stroke};

use crate::theme;

/// Stylized shield with a checkmark inside — consent-screen hero icon.
/// `fill` paints the shield body; the check is rendered in white.
pub fn shield(p: &Painter, center: Pos2, size: f32, fill: Color32) {
    let half = size * 0.5;
    let pts = vec![
        center + vec2(-half * 0.85, -half * 0.92),
        center + vec2( half * 0.85, -half * 0.92),
        center + vec2( half * 0.85,  half * 0.10),
        center + vec2( 0.0,           half * 0.95),
        center + vec2(-half * 0.85,  half * 0.10),
    ];
    p.add(Shape::convex_polygon(pts, fill, Stroke::NONE));

    let stroke = Stroke::new((size * 0.10).max(2.0), Color32::WHITE);
    let a = center + vec2(-half * 0.32, -half * 0.04);
    let b = center + vec2(-half * 0.05,  half * 0.28);
    let c = center + vec2( half * 0.40, -half * 0.26);
    p.line_segment([a, b], stroke);
    p.line_segment([b, c], stroke);
}

/// User glyph — head circle plus shoulders curve, used in info rows.
pub fn person(p: &Painter, center: Pos2, size: f32, color: Color32) {
    let half = size * 0.5;
    p.circle_filled(center + vec2(0.0, -half * 0.32), half * 0.30, color);
    let torso = vec![
        center + vec2(-half * 0.55, half * 0.6),
        center + vec2(-half * 0.50, half * 0.10),
        center + vec2(-half * 0.20, -half * 0.05),
        center + vec2( half * 0.20, -half * 0.05),
        center + vec2( half * 0.50, half * 0.10),
        center + vec2( half * 0.55, half * 0.6),
    ];
    p.add(Shape::convex_polygon(torso, color, Stroke::NONE));
}

/// Clock — outline circle with two hands (12 + 3 o'clock).
pub fn clock(p: &Painter, center: Pos2, size: f32, color: Color32) {
    let half = size * 0.5;
    let stroke = Stroke::new((size * 0.10).max(1.5), color);
    p.circle_stroke(center, half * 0.85, stroke);
    p.line_segment([center, center + vec2(0.0, -half * 0.45)], stroke);
    p.line_segment([center, center + vec2(half * 0.55, 0.0)], stroke);
    p.circle_filled(center, (size * 0.07).max(1.0), color);
}

/// Lower-case `i` inside an outlined circle.
pub fn info(p: &Painter, center: Pos2, size: f32, color: Color32) {
    let half = size * 0.5;
    let stroke = Stroke::new((size * 0.10).max(1.5), color);
    p.circle_stroke(center, half * 0.85, stroke);
    p.circle_filled(center + vec2(0.0, -half * 0.30), (size * 0.08).max(1.0), color);
    p.line_segment(
        [center + vec2(0.0, -half * 0.05), center + vec2(0.0, half * 0.40)],
        Stroke::new((size * 0.13).max(2.0), color),
    );
}

/// Eye — outline almond + inner pupil. Used to convey "view your screen".
pub fn eye(p: &Painter, center: Pos2, size: f32, color: Color32) {
    let half = size * 0.5;
    let stroke = Stroke::new((size * 0.10).max(1.5), color);
    let almond = vec![
        center + vec2(-half * 0.85,  0.0),
        center + vec2(-half * 0.40, -half * 0.45),
        center + vec2( half * 0.40, -half * 0.45),
        center + vec2( half * 0.85,  0.0),
        center + vec2( half * 0.40,  half * 0.45),
        center + vec2(-half * 0.40,  half * 0.45),
    ];
    p.add(Shape::closed_line(almond, stroke));
    p.circle_filled(center, half * 0.28, color);
}

/// Paper-plane glyph for the chat send button.
pub fn send(p: &Painter, center: Pos2, size: f32, color: Color32) {
    let half = size * 0.5;
    let pts = vec![
        center + vec2(-half * 0.78, -half * 0.55),
        center + vec2( half * 0.78,  0.0),
        center + vec2(-half * 0.78,  half * 0.55),
        center + vec2(-half * 0.30,  0.0),
    ];
    p.add(Shape::convex_polygon(pts, color, Stroke::NONE));
}

/// X / close — two crossing strokes with rounded line ends.
pub fn close_x(p: &Painter, center: Pos2, size: f32, color: Color32) {
    let half = size * 0.32;
    let stroke = Stroke::new((size * 0.13).max(1.5), color);
    p.line_segment([center + vec2(-half, -half), center + vec2(half,  half)], stroke);
    p.line_segment([center + vec2(-half,  half), center + vec2(half, -half)], stroke);
}

/// Solid circle whose alpha is driven by the caller — used for the live-session
/// pulse indicator. Includes a subtle outer ring so a low-alpha pulse still
/// reads on white backgrounds.
pub fn pulse_dot(p: &Painter, center: Pos2, size: f32, base: Color32, alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    let inner = Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), (a * 255.0) as u8);
    let halo  = Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), (a * 60.0)  as u8);
    p.circle_filled(center, size * 0.55, halo);
    p.circle_filled(center, size * 0.32, inner);
}

/// Avatar pill — flat circle with the first letter of `name` in white. Falls
/// back to a question mark when `name` is empty.
pub fn avatar(p: &Painter, center: Pos2, size: f32, fill: Color32, name: &str) {
    p.circle_filled(center, size * 0.5, fill);
    let initial = name
        .trim()
        .chars()
        .next()
        .map(|c| c.to_uppercase().next().unwrap_or(c))
        .unwrap_or('?');
    p.text(
        center + vec2(0.0, -size * 0.04),
        egui::Align2::CENTER_CENTER,
        initial.to_string(),
        egui::FontId::proportional(size * 0.46),
        Color32::WHITE,
    );
}

/// Horizontal progress bar — track + fill rounded to half-height pill ends.
/// `fraction` is clamped to [0, 1]; pass it as remaining/total (drains over time)
/// or done/total (fills over time) depending on context.
pub fn progress_bar(p: &Painter, rect: Rect, fraction: f32, fill: Color32) {
    let radius = rect.height() * 0.5;
    p.rect_filled(rect, Rounding::same(radius), theme::BORDER);
    let f = fraction.clamp(0.0, 1.0);
    if f > 0.0 {
        let filled = Rect::from_min_size(rect.min, vec2(rect.width() * f, rect.height()));
        p.rect_filled(filled, Rounding::same(radius), fill);
    }
}

/// Circular countdown timer — gray track ring, colored draining arc, centered number.
/// `fraction` is remaining/total (1.0 = full, 0.0 = empty).
pub fn circular_countdown(
    p: &Painter,
    center: Pos2,
    size: f32,
    fraction: f32,
    color: Color32,
    remaining_secs: u64,
) {
    let radius = size * 0.40;
    let track_w = (size * 0.095).max(2.5);

    // Gray background track
    p.circle_stroke(
        center,
        radius,
        Stroke::new(track_w, Color32::from_rgb(0xE5, 0xE7, 0xEB)),
    );

    // Colored arc draining clockwise from 12 o'clock
    let f = fraction.clamp(0.0, 1.0);
    if f > 0.005 {
        let steps = 64usize;
        let n = ((f * steps as f32).ceil() as usize).max(2);
        let start = -std::f32::consts::FRAC_PI_2;
        let sweep = f * std::f32::consts::TAU;
        let mut pts: Vec<Pos2> = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let angle = start + (i as f32 / n as f32) * sweep;
            pts.push(center + vec2(angle.cos() * radius, angle.sin() * radius));
        }
        p.add(Shape::line(pts, Stroke::new(track_w, color)));
    }

    // Remaining seconds centered
    p.text(
        center,
        egui::Align2::CENTER_CENTER,
        remaining_secs.to_string(),
        egui::FontId::proportional(size * 0.30),
        color,
    );
}

/// Mouse outline icon — represents "Control mouse & keyboard".
pub fn mouse_cursor(p: &Painter, center: Pos2, size: f32, color: Color32) {
    let half = size * 0.5;
    let stroke = Stroke::new((size * 0.09).max(1.5), color);
    let bw = half * 0.52;
    let bh = half * 0.88;
    let body = Rect::from_center_size(center + vec2(0.0, half * 0.08), vec2(bw * 2.0, bh * 2.0));
    p.rect_stroke(body, Rounding::same(bw), stroke);
    // Left/right button divider from top-center downward
    let top_mid = Pos2::new(center.x, body.min.y);
    let div_end = Pos2::new(center.x, body.min.y + bh * 0.60);
    p.line_segment([top_mid, div_end], stroke);
}

/// Two vertical arrows (down-left, up-right) — represents file download/upload.
pub fn file_transfer(p: &Painter, center: Pos2, size: f32, color: Color32) {
    let half = size * 0.5;
    let stroke = Stroke::new((size * 0.10).max(1.5), color);
    let ah = (size * 0.13).max(2.0);
    let gap = half * 0.28;

    // Down arrow (left side)
    let lx = center.x - gap;
    let l_top = Pos2::new(lx, center.y - half * 0.60);
    let l_bot = Pos2::new(lx, center.y + half * 0.28);
    p.line_segment([l_top, l_bot], stroke);
    p.line_segment([l_bot, Pos2::new(lx - ah, center.y + half * 0.02)], stroke);
    p.line_segment([l_bot, Pos2::new(lx + ah, center.y + half * 0.02)], stroke);

    // Up arrow (right side)
    let rx = center.x + gap;
    let r_top = Pos2::new(rx, center.y - half * 0.28);
    let r_bot = Pos2::new(rx, center.y + half * 0.60);
    p.line_segment([r_top, r_bot], stroke);
    p.line_segment([r_top, Pos2::new(rx - ah, center.y - half * 0.02)], stroke);
    p.line_segment([r_top, Pos2::new(rx + ah, center.y - half * 0.02)], stroke);
}

/// Padlock — shackle arc above a rounded rectangular body.
/// Used on the end-to-end encryption note.
pub fn lock(p: &Painter, center: Pos2, size: f32, color: Color32) {
    let half = size * 0.5;
    let stroke = Stroke::new((size * 0.09).max(1.5), color);
    let bw = half * 0.62;
    let bh = half * 0.48;
    let body_top = center.y + half * 0.04;

    // Body rectangle
    let body = Rect::from_min_max(
        Pos2::new(center.x - bw, body_top),
        Pos2::new(center.x + bw, body_top + bh * 2.0),
    );
    p.rect_stroke(body, Rounding::same(half * 0.14), stroke);

    // Shackle legs + semicircle arc
    let sw = bw * 0.56;
    let arc_cy = body_top - sw * 0.80;
    p.line_segment(
        [Pos2::new(center.x - sw, body_top), Pos2::new(center.x - sw, arc_cy)],
        stroke,
    );
    p.line_segment(
        [Pos2::new(center.x + sw, body_top), Pos2::new(center.x + sw, arc_cy)],
        stroke,
    );
    let steps = 20usize;
    let mut arc_pts: Vec<Pos2> = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let angle = std::f32::consts::PI
            + (i as f32 / steps as f32) * std::f32::consts::PI;
        arc_pts.push(Pos2::new(
            center.x + angle.cos() * sw,
            arc_cy + angle.sin() * sw,
        ));
    }
    p.add(Shape::line(arc_pts, stroke));

    // Keyhole dot
    p.circle_filled(
        Pos2::new(center.x, body_top + bh * 0.90),
        (size * 0.07).max(1.0),
        color,
    );
}
