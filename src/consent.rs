use std::time::{Duration, Instant};

use eframe::{egui, App, NativeOptions};
use egui::{Align, Align2, Color32, FontFamily, FontId, Frame, Layout, Margin, RichText, Sense, Stroke};

use crate::icons;
use crate::theme::{
    self, with_alpha, BORDER, BORDER_STRONG, DANGER, PRIMARY, PRIMARY_HOVER, PRIMARY_PRESS,
    SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS, SURFACE, SURFACE_ALT, TEXT,
    TEXT_MUTED, TEXT_SUBTLE, WARNING, FONT_SEMIBOLD,
};
use crate::timer::Countdown;

const WIN_W: f32 = 460.0;
const WIN_H: f32 = 490.0;

pub fn run(
    admin_name: String,
    admin_email: String,
    session_id: String,
    timeout: u64,
) -> Result<(), eframe::Error> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("Remote Access Request")
        .with_inner_size([WIN_W, WIN_H])
        .with_min_inner_size([WIN_W, WIN_H])
        .with_max_inner_size([WIN_W, WIN_H])
        .with_resizable(false)
        .with_maximize_button(false)
        .with_minimize_button(false)
        .with_always_on_top();

    let options = NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Remote Access Request",
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            Box::new(ConsentApp::new(admin_name, admin_email, session_id, timeout))
        }),
    )
}

#[derive(Clone, Copy, PartialEq)]
enum Decision {
    Accepted,
    Declined,
}

struct ConsentApp {
    admin_name: String,
    admin_email: String,
    session_id: String,
    countdown: Countdown,
    countdown_total: u64,
    decision: Option<Decision>,
    closing_at: Option<Instant>,
    output_written: bool,
}

impl ConsentApp {
    fn new(admin_name: String, admin_email: String, session_id: String, timeout: u64) -> Self {
        Self {
            admin_name,
            admin_email,
            session_id,
            countdown: Countdown::new(timeout),
            countdown_total: timeout.max(1),
            decision: None,
            closing_at: None,
            output_written: false,
        }
    }

    fn write_output(&mut self) {
        if self.output_written {
            return;
        }
        self.output_written = true;
        let result = match self.decision {
            Some(Decision::Accepted) => "accepted",
            _ => "declined",
        };
        println!(
            r#"{{"result":"{}","session_id":"{}"}}"#,
            result,
            escape_json(&self.session_id)
        );
    }

    fn record(&mut self, d: Decision) {
        if self.decision.is_some() {
            return;
        }
        self.decision = Some(d);
        self.write_output();
        self.closing_at = Some(Instant::now() + Duration::from_millis(180));
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

impl App for ConsentApp {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        // Pure white root paints seamlessly under the OS title bar on every
        // platform — avoids the visible seam that a tinted body would create.
        [1.0, 1.0, 1.0, 1.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) && self.decision.is_none() {
            self.record(Decision::Declined);
        }

        if let Some(t) = self.closing_at {
            if Instant::now() >= t {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        if self.decision.is_none() && self.countdown.expired() {
            self.record(Decision::Accepted);
        }

        ctx.request_repaint_after(Duration::from_millis(150));

        let remaining = self.countdown.remaining_secs();
        let countdown_color = if remaining <= 5 {
            DANGER
        } else if remaining <= 15 {
            WARNING
        } else {
            PRIMARY
        };

        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(SURFACE)
                    .inner_margin(Margin {
                        left: SPACE_XL + 4.0,
                        right: SPACE_XL + 4.0,
                        top: SPACE_XL,
                        bottom: SPACE_LG,
                    }),
            )
            .show(ctx, |ui| {
                // ── Header: circular timer + title/subtitle ───────────────
                ui.horizontal(|ui| {
                    let timer_size = 58.0;
                    let (tr, _) =
                        ui.allocate_exact_size(egui::vec2(timer_size, timer_size), Sense::hover());
                    let frac = (remaining as f32) / (self.countdown_total as f32).max(1.0);
                    icons::circular_countdown(
                        ui.painter(),
                        tr.center(),
                        timer_size,
                        frac,
                        countdown_color,
                        remaining,
                    );

                    ui.add_space(SPACE_MD + 2.0);

                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 5.0;
                        ui.label(
                            RichText::new("Remote Access Request")
                                .font(FontId::new(19.0, FontFamily::Name(FONT_SEMIBOLD.into())))
                                .color(Color32::from_rgb(0x0D, 0x14, 0x21)),
                        );
                        ui.label(
                            RichText::new(
                                "A technician is requesting permission to view and \
                                 control your screen.",
                            )
                            .size(12.5)
                            .color(TEXT_SUBTLE),
                        );
                        ui.label(
                            RichText::new("Approve only if you initiated this request.")
                                .size(12.0)
                                .color(TEXT_MUTED),
                        );
                    });
                });

                ui.add_space(SPACE_LG);

                // ── Admin identity card ───────────────────────────────────
                Frame::none()
                    .fill(SURFACE_ALT)
                    .stroke(Stroke::new(1.0, BORDER))
                    .rounding(theme::r_md())
                    .inner_margin(Margin::symmetric(SPACE_LG, SPACE_SM + 4.0))
                    .show(ui, |ui| {
                        // Force the frame to stretch full width
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let (ar, _) =
                                ui.allocate_exact_size(egui::vec2(38.0, 38.0), Sense::hover());
                            icons::avatar(
                                ui.painter(),
                                ar.center(),
                                38.0,
                                PRIMARY,
                                &self.admin_name,
                            );

                            ui.add_space(SPACE_SM + 2.0);

                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 3.0;
                                ui.label(
                                    RichText::new(&self.admin_name)
                                        .font(FontId::new(14.5, FontFamily::Name(FONT_SEMIBOLD.into())))
                                        .color(TEXT),
                                );
                                if !self.admin_email.is_empty() {
                                    ui.label(
                                        RichText::new(&self.admin_email)
                                            .size(12.0)
                                            .color(TEXT_SUBTLE),
                                    );
                                }
                            });
                        });
                    });

                ui.add_space(SPACE_LG);

                // ── Permissions list ──────────────────────────────────────
                ui.label(
                    RichText::new("PERMISSIONS REQUESTED")
                        .font(FontId::new(11.0, FontFamily::Name(FONT_SEMIBOLD.into())))
                        .color(TEXT_SUBTLE),
                );
                ui.add_space(SPACE_SM);

                permission_row(
                    ui,
                    |p, c, s| icons::eye(p, c, s, PRIMARY),
                    "View your screen",
                );
                ui.add_space(SPACE_XS);
                permission_row(
                    ui,
                    |p, c, s| icons::mouse_cursor(p, c, s, PRIMARY),
                    "Control mouse & keyboard",
                );
                ui.add_space(SPACE_XS);
                permission_row(
                    ui,
                    |p, c, s| icons::file_transfer(p, c, s, PRIMARY),
                    "Can download/upload files from/to the system",
                );

                ui.add_space(SPACE_LG);

                // ── Encryption note ───────────────────────────────────────
                Frame::none()
                    .fill(theme::PRIMARY_TINT)
                    .stroke(Stroke::new(1.0, with_alpha(PRIMARY, 45)))
                    .rounding(theme::r_md())
                    .inner_margin(Margin::symmetric(SPACE_MD, SPACE_SM + 2.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (lr, _) =
                                ui.allocate_exact_size(egui::vec2(16.0, 16.0), Sense::hover());
                            icons::lock(ui.painter(), lr.center(), 14.0, theme::PRIMARY_INK);
                            ui.add_space(SPACE_SM);
                            ui.label(
                                RichText::new(
                                    "Session is end-to-end encrypted and audit-logged.",
                                )
                                .size(12.5)
                                .color(theme::PRIMARY_INK),
                            );
                        });
                    });

                ui.add_space(SPACE_LG + 4.0);

                // ── Action row ────────────────────────────────────────────
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if primary_button(ui, "Allow Access", 152.0).clicked() {
                            self.record(Decision::Accepted);
                        }
                        ui.add_space(SPACE_SM + 2.0);
                        if secondary_button(ui, "Decline", 100.0).clicked() {
                            self.record(Decision::Declined);
                        }
                    });
                });

                // ── Footer ────────────────────────────────────────────────
                let footer_h = 14.0;
                let remaining_space = ui.available_height() - footer_h;
                if remaining_space > 0.0 {
                    ui.add_space(remaining_space);
                }
                ui.horizontal(|ui| {
                    let (mark_rect, _) =
                        ui.allocate_exact_size(egui::vec2(12.0, 12.0), Sense::hover());
                    icons::shield(ui.painter(), mark_rect.center(), 11.0, theme::TEXT_FAINT);
                    ui.add_space(SPACE_XS + 2.0);
                    ui.label(
                        RichText::new("SecOps Solution")
                            .size(10.5)
                            .color(theme::TEXT_FAINT)
                            .strong(),
                    );
                    ui.label(RichText::new("·").size(10.5).color(theme::TEXT_FAINT));
                    ui.label(
                        RichText::new("Managed by your IT department")
                            .size(10.5)
                            .color(theme::TEXT_FAINT),
                    );
                });
            });
    }
}

// ── Permission row helper ──────────────────────────────────────────────────

fn permission_row(
    ui: &mut egui::Ui,
    draw_icon: impl FnOnce(&egui::Painter, egui::Pos2, f32),
    text: &str,
) {
    ui.horizontal(|ui| {
        let (ir, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), Sense::hover());
        draw_icon(ui.painter(), ir.center(), 15.0);
        ui.add_space(SPACE_SM);
        ui.label(RichText::new(text).size(13.5).color(TEXT));
    });
}

// ── Buttons ────────────────────────────────────────────────────────────────

fn primary_button(ui: &mut egui::Ui, label: &str, width: f32) -> egui::Response {
    let height = 36.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, height), Sense::click());

    let bg = if response.is_pointer_button_down_on() {
        PRIMARY_PRESS
    } else if response.hovered() {
        PRIMARY_HOVER
    } else {
        PRIMARY
    };

    // Subtle shadow on hover gives a "lift" cue without drama.
    if response.hovered() && !response.is_pointer_button_down_on() {
        let painter = ui.painter();
        painter.rect_filled(
            rect.translate(egui::vec2(0.0, 1.5)),
            theme::r_md(),
            with_alpha(PRIMARY, 30),
        );
    }

    let painter = ui.painter();
    painter.rect_filled(rect, theme::r_md(), bg);
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(13.5),
        Color32::WHITE,
    );

    if response.has_focus() {
        painter.rect_stroke(
            rect.expand(2.0),
            theme::r_md(),
            Stroke::new(2.0, with_alpha(PRIMARY, 90)),
        );
    }
    response
}

fn secondary_button(ui: &mut egui::Ui, label: &str, width: f32) -> egui::Response {
    let height = 36.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, height), Sense::click());

    let bg = if response.is_pointer_button_down_on() {
        theme::SURFACE_PRESS
    } else if response.hovered() {
        theme::SURFACE_HOVER
    } else {
        SURFACE
    };
    let border = if response.hovered() {
        BORDER_STRONG
    } else {
        BORDER
    };

    let painter = ui.painter();
    painter.rect_filled(rect, theme::r_md(), bg);
    painter.rect_stroke(rect, theme::r_md(), Stroke::new(1.0, border));
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(13.5),
        TEXT,
    );
    response
}

