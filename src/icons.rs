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
