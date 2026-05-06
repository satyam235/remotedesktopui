// Enterprise-grade light theme. Palette and rhythm follow modern enterprise
// design conventions (Linear / Stripe / GitHub Primer): cool off-white
// backgrounds, restrained brand blue, generous whitespace, soft shadows,
// subtle 1px borders.
//
// This module exposes a complete design-system surface — some constants and
// helpers are intentionally unused at the call sites below so callers can
// reach for them without repeated round-trips to this file.
#![allow(dead_code)]

use egui::{epaint::Shadow, Color32, FontFamily, FontId, Margin, Rounding, Stroke, TextStyle, Visuals};

// ── Surfaces ────────────────────────────────────────────────────────────────
pub const BG:           Color32 = Color32::from_rgb(0xF7, 0xF8, 0xFA); // app background
pub const SURFACE:      Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF); // cards / dialogs
pub const SURFACE_ALT:  Color32 = Color32::from_rgb(0xF3, 0xF5, 0xF7); // info cards / inputs
pub const SURFACE_HOVER:Color32 = Color32::from_rgb(0xEC, 0xEF, 0xF3); // button hover
pub const SURFACE_PRESS:Color32 = Color32::from_rgb(0xE2, 0xE6, 0xEC); // button press

// ── Text ────────────────────────────────────────────────────────────────────
pub const TEXT:         Color32 = Color32::from_rgb(0x0F, 0x17, 0x24); // primary
pub const TEXT_SUBTLE:  Color32 = Color32::from_rgb(0x37, 0x41, 0x51); // secondary
pub const TEXT_MUTED:   Color32 = Color32::from_rgb(0x6B, 0x72, 0x80); // tertiary / labels
pub const TEXT_FAINT:   Color32 = Color32::from_rgb(0x9C, 0xA3, 0xAF); // disabled

// ── Borders ─────────────────────────────────────────────────────────────────
pub const BORDER:        Color32 = Color32::from_rgb(0xE5, 0xE7, 0xEB); // hairline
pub const BORDER_STRONG: Color32 = Color32::from_rgb(0xD1, 0xD5, 0xDB); // emphasized

// ── Brand ───────────────────────────────────────────────────────────────────
pub const PRIMARY:       Color32 = Color32::from_rgb(0x25, 0x63, 0xEB); // trust blue
pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(0x1D, 0x4E, 0xD8);
pub const PRIMARY_PRESS: Color32 = Color32::from_rgb(0x1E, 0x40, 0xAF);
pub const PRIMARY_TINT:  Color32 = Color32::from_rgb(0xEF, 0xF6, 0xFF); // tinted bg
pub const PRIMARY_INK:   Color32 = Color32::from_rgb(0x1E, 0x40, 0xAF); // text on tint

// ── Status ──────────────────────────────────────────────────────────────────
pub const SUCCESS:       Color32 = Color32::from_rgb(0x05, 0x96, 0x69);
pub const SUCCESS_TINT:  Color32 = Color32::from_rgb(0xEC, 0xFD, 0xF5);
pub const DANGER:        Color32 = Color32::from_rgb(0xDC, 0x26, 0x26);
pub const DANGER_HOVER:  Color32 = Color32::from_rgb(0xB9, 0x1C, 0x1C);
pub const DANGER_TINT:   Color32 = Color32::from_rgb(0xFE, 0xF2, 0xF2);
pub const WARNING:       Color32 = Color32::from_rgb(0xD9, 0x77, 0x06);
pub const WARNING_TINT:  Color32 = Color32::from_rgb(0xFF, 0xFB, 0xEB);

// ── Shadows ─────────────────────────────────────────────────────────────────
/// Soft drop shadow for top-level cards (consent dialog, session overlay).
pub fn shadow_card() -> Shadow {
    Shadow {
        offset: egui::vec2(0.0, 4.0),
        blur:   24.0,
        spread: 0.0,
        color:  Color32::from_rgba_unmultiplied(15, 23, 36, 28),
    }
}

/// Subtle shadow for raised inline elements (popovers, focused inputs).
pub fn shadow_subtle() -> Shadow {
    Shadow {
        offset: egui::vec2(0.0, 1.0),
        blur:   3.0,
        spread: 0.0,
        color:  Color32::from_rgba_unmultiplied(15, 23, 36, 18),
    }
}

// ── Spacing scale (px) ──────────────────────────────────────────────────────
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
pub const SPACE_LG: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;

// ── Radii ───────────────────────────────────────────────────────────────────
pub const RADIUS_SM: f32 = 6.0;
pub const RADIUS_MD: f32 = 8.0;
pub const RADIUS_LG: f32 = 12.0;

pub fn r_sm() -> Rounding { Rounding::same(RADIUS_SM) }
pub fn r_md() -> Rounding { Rounding::same(RADIUS_MD) }
pub fn r_lg() -> Rounding { Rounding::same(RADIUS_LG) }

// ── Typography ──────────────────────────────────────────────────────────────
// egui ships only one weight per font family; we lean on size + .strong()
// (renders bold) + tracked uppercase labels for hierarchy. The ratios below
// are tuned for crispness on both 1x and 2x displays.
pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles.insert(TextStyle::Heading,    FontId::new(20.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Body,       FontId::new(13.5, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Button,     FontId::new(13.5, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Small,      FontId::new(11.5, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Monospace,  FontId::new(12.5, FontFamily::Monospace));

    let mut v = Visuals::light();

    v.override_text_color   = Some(TEXT);
    v.window_fill           = SURFACE;
    v.panel_fill            = BG;
    v.faint_bg_color        = SURFACE_ALT;
    v.extreme_bg_color      = SURFACE;
    v.window_stroke         = Stroke::new(1.0, BORDER);
    v.window_shadow         = shadow_card();
    v.window_rounding       = r_lg();
    v.popup_shadow          = shadow_subtle();
    v.menu_rounding         = r_md();

    let r = r_md();

    v.widgets.noninteractive.bg_fill      = SURFACE;
    v.widgets.noninteractive.weak_bg_fill = SURFACE;
    v.widgets.noninteractive.bg_stroke    = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.fg_stroke    = Stroke::new(1.0, TEXT);
    v.widgets.noninteractive.rounding     = r;
    v.widgets.noninteractive.expansion    = 0.0;

    v.widgets.inactive.bg_fill            = SURFACE;
    v.widgets.inactive.weak_bg_fill       = SURFACE_ALT;
    v.widgets.inactive.bg_stroke          = Stroke::new(1.0, BORDER);
    v.widgets.inactive.fg_stroke          = Stroke::new(1.0, TEXT);
    v.widgets.inactive.rounding           = r;
    v.widgets.inactive.expansion          = 0.0;

    v.widgets.hovered.bg_fill             = SURFACE_HOVER;
    v.widgets.hovered.weak_bg_fill        = SURFACE_HOVER;
    v.widgets.hovered.bg_stroke           = Stroke::new(1.0, BORDER_STRONG);
    v.widgets.hovered.fg_stroke           = Stroke::new(1.0, TEXT);
    v.widgets.hovered.rounding            = r;
    v.widgets.hovered.expansion           = 0.0;

    v.widgets.active.bg_fill              = SURFACE_PRESS;
    v.widgets.active.weak_bg_fill         = SURFACE_PRESS;
    v.widgets.active.bg_stroke            = Stroke::new(1.5, PRIMARY);
    v.widgets.active.fg_stroke            = Stroke::new(1.0, TEXT);
    v.widgets.active.rounding             = r;
    v.widgets.active.expansion            = 0.0;

    v.selection.bg_fill = with_alpha(PRIMARY, 50);
    v.selection.stroke  = Stroke::new(1.0, PRIMARY);
    v.hyperlink_color   = PRIMARY;

    style.visuals = v;
    style.spacing.item_spacing   = egui::vec2(SPACE_SM, SPACE_SM);
    style.spacing.button_padding = egui::vec2(SPACE_MD, SPACE_SM);
    style.spacing.window_margin  = Margin::same(0.0);
    style.spacing.menu_margin    = Margin::same(SPACE_XS);

    ctx.set_style(style);
}

// ── Color helpers ───────────────────────────────────────────────────────────

pub fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

pub fn darken(c: Color32, amount: f32) -> Color32 {
    let f = (1.0 - amount).clamp(0.0, 1.0);
    Color32::from_rgb(
        (c.r() as f32 * f) as u8,
        (c.g() as f32 * f) as u8,
        (c.b() as f32 * f) as u8,
    )
}

pub fn lighten(c: Color32, amount: f32) -> Color32 {
    let a = amount.clamp(0.0, 1.0);
    Color32::from_rgb(
        (c.r() as f32 + (255.0 - c.r() as f32) * a) as u8,
        (c.g() as f32 + (255.0 - c.g() as f32) * a) as u8,
        (c.b() as f32 + (255.0 - c.b() as f32) * a) as u8,
    )
}
