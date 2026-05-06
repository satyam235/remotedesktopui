use std::time::{Duration, Instant};

use eframe::{egui, App, NativeOptions};
use egui::{Align, Color32, Frame, Layout, Margin, RichText, Rounding, Sense, Stroke};

use crate::chat::{self, ChatHandle, Message};
use crate::icons;
use crate::theme::{
    self, with_alpha, BG, BORDER, DANGER, PRIMARY, PRIMARY_INK, PRIMARY_TINT, SPACE_LG, SPACE_MD,
    SPACE_SM, SPACE_XS, SUCCESS, SURFACE, SURFACE_ALT, TEXT, TEXT_FAINT, TEXT_MUTED, TEXT_SUBTLE,
};
use crate::timer::{format_clock, unix_ms, Elapsed};

const WIN_W: f32 = 360.0;
const WIN_H: f32 = 580.0;

pub fn run(
    admin_name: String,
    session_id: String,
    chat_socket: String,
) -> Result<(), eframe::Error> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("SecOps Session")
        .with_inner_size([WIN_W, WIN_H])
        .with_min_inner_size([320.0, 420.0])
        .with_resizable(true)
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top();

    let options = NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "SecOps Session",
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            Box::new(SessionApp::new(admin_name, session_id, chat_socket))
        }),
    )
}

struct ChatLine {
    from: String,
    text: String,
    ts: u64,
}

struct Toast {
    text: String,
    until: Instant,
    danger: bool,
}

struct SessionApp {
    admin_name: String,
    session_id: String,
    chat: ChatHandle,
    history: Vec<ChatLine>,
    input: String,
    elapsed: Elapsed,
    toast: Option<Toast>,
    closing_at: Option<Instant>,
    pending_focus: bool,
    positioned: bool,
}

impl SessionApp {
    fn new(admin_name: String, session_id: String, chat_socket: String) -> Self {
        let chat = chat::start(chat_socket);
        Self {
            admin_name,
            session_id,
            chat,
            history: Vec::new(),
            input: String::new(),
            elapsed: Elapsed::new(),
            toast: None,
            closing_at: None,
            pending_focus: true,
            positioned: false,
        }
    }

    fn send_user_text(&mut self) {
        let text = self.input.trim().to_string();
        if text.is_empty() {
            return;
        }
        let ts = unix_ms();
        self.chat.send(Message {
            kind: "chat".to_string(),
            session_id: self.session_id.clone(),
            from: "user".to_string(),
            text: text.clone(),
            ts,
        });
        self.history.push(ChatLine {
            from: "user".to_string(),
            text,
            ts,
        });
        self.input.clear();
        self.pending_focus = true;
    }

    fn disconnect(&mut self) {
        if self.closing_at.is_some() {
            return;
        }
        self.chat.send(Message {
            kind: "disconnect".to_string(),
            session_id: self.session_id.clone(),
            from: "user".to_string(),
            text: String::new(),
            ts: unix_ms(),
        });
        self.toast = Some(Toast {
            text: "Disconnecting…".to_string(),
            until: Instant::now() + Duration::from_millis(1100),
            danger: false,
        });
        self.closing_at = Some(Instant::now() + Duration::from_secs(1));
    }
}

impl App for SessionApp {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Place the window in the top-right corner of the primary monitor on
        // first paint, once the window manager has reported its size.
        if !self.positioned {
            let monitor = ctx.input(|i| i.viewport().monitor_size);
            if let Some(m) = monitor {
                if m.x > 400.0 && m.y > 100.0 {
                    let pos = egui::pos2(m.x - WIN_W - 24.0, 24.0);
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
                    self.positioned = true;
                }
            }
        }

        ctx.request_repaint_after(Duration::from_millis(33));

        for m in self.chat.poll() {
            match m.kind.as_str() {
                "chat" => self.history.push(ChatLine {
                    from: if m.from.is_empty() { "admin".into() } else { m.from },
                    text: m.text,
                    ts: if m.ts > 0 { m.ts } else { unix_ms() },
                }),
                "session_end" => {
                    self.toast = Some(Toast {
                        text: "Session ended by technician".into(),
                        until: Instant::now() + Duration::from_secs(3),
                        danger: true,
                    });
                    self.closing_at = Some(Instant::now() + Duration::from_secs(3));
                }
                _ => {}
            }
        }

        if let Some(t) = self.closing_at {
            if Instant::now() >= t {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        if ctx.input(|i| i.viewport().close_requested()) && self.closing_at.is_none() {
            self.disconnect();
        }

        // ── Frame: white card with rounded corners + shadow on transparent root ──
        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(SURFACE)
                    .rounding(Rounding::same(theme::RADIUS_LG))
                    .stroke(Stroke::new(1.0, BORDER))
                    .shadow(theme::shadow_card())
                    .inner_margin(Margin::same(0.0)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                let want_disconnect = draw_header(ui, ctx, &self.admin_name);
                if want_disconnect {
                    self.disconnect();
                }

                // Reserve enough room for: top hairline (1) + composer (38) +
                // composer→status gap (8) + status row (18) + Frame top/bottom
                // inner_margin (10 + 10) + a small safety buffer for DPI rounding.
                // Using a fixed reservation keeps the chat area stable when the
                // status row text changes ("Connecting…" → "Connected").
                let footer_h = 110.0;
                let avail = ui.available_size();
                let chat_h = (avail.y - footer_h).max(120.0);

                // ── Chat scroll body ──────────────────────────────────────
                Frame::none()
                    .fill(BG)
                    .inner_margin(Margin::symmetric(SPACE_MD, SPACE_MD))
                    .show(ui, |ui| {
                        ui.set_min_height(chat_h);
                        ui.set_max_height(chat_h);

                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(4.0, 10.0);

                                if self.history.is_empty() {
                                    draw_empty_state(ui);
                                } else {
                                    let mut last_from = String::new();
                                    for line in &self.history {
                                        let same_speaker =
                                            line.from.eq_ignore_ascii_case(&last_from);
                                        if !same_speaker {
                                            ui.add_space(2.0);
                                        }
                                        draw_bubble(ui, line, !same_speaker, &self.admin_name);
                                        last_from = line.from.clone();
                                    }
                                }
                            });
                    });

                // ── Footer (composer + status) ────────────────────────────
                // Top hairline drawn explicitly inside the Frame so it always
                // sits exactly above the composer, regardless of layout flow.
                Frame::none()
                    .fill(SURFACE)
                    .inner_margin(Margin {
                        left: SPACE_MD,
                        right: SPACE_MD,
                        top: SPACE_SM + 2.0,
                        bottom: SPACE_SM + 2.0,
                    })
                    .show(ui, |ui| {
                        let frame_top_left = ui.min_rect().min;
                        ui.painter().line_segment(
                            [
                                egui::pos2(frame_top_left.x - SPACE_MD, frame_top_left.y - SPACE_SM - 2.0),
                                egui::pos2(
                                    frame_top_left.x - SPACE_MD + WIN_W,
                                    frame_top_left.y - SPACE_SM - 2.0,
                                ),
                            ],
                            Stroke::new(1.0, BORDER),
                        );

                        let response = composer(ui, &mut self.input, self.pending_focus);
                        let enter_pressed = response.field_response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        let should_send = (enter_pressed || response.send_clicked)
                            && !self.input.trim().is_empty();
                        if should_send {
                            self.send_user_text();
                            response.field_response.request_focus();
                        } else if self.pending_focus {
                            response.field_response.request_focus();
                            self.pending_focus = false;
                        }

                        ui.add_space(SPACE_SM + 2.0);
                        draw_status_strip(ui, &self.elapsed, &self.chat);
                    });
            });

        // ── Toast overlay ─────────────────────────────────────────────────
        if let Some(t) = &self.toast {
            if Instant::now() >= t.until {
                self.toast = None;
            } else {
                let screen = ctx.screen_rect();
                let area_id = egui::Id::new("session_toast");
                let toast_w = 240.0;
                egui::Area::new(area_id)
                    .order(egui::Order::Foreground)
                    .fixed_pos(egui::pos2(
                        screen.center().x - toast_w * 0.5,
                        screen.center().y - 24.0,
                    ))
                    .show(ctx, |ui| {
                        let bg = if t.danger { theme::DANGER_TINT } else { SURFACE };
                        let stroke = if t.danger { Stroke::new(1.0, DANGER) } else { Stroke::new(1.0, BORDER) };
                        Frame::none()
                            .fill(bg)
                            .stroke(stroke)
                            .rounding(theme::r_md())
                            .shadow(theme::shadow_card())
                            .inner_margin(Margin::symmetric(SPACE_LG, SPACE_MD))
                            .show(ui, |ui| {
                                ui.set_min_width(toast_w);
                                ui.label(
                                    RichText::new(&t.text)
                                        .size(13.0)
                                        .color(if t.danger { DANGER } else { TEXT })
                                        .strong(),
                                );
                            });
                    });
            }
        }
    }
}

// ── Header ─────────────────────────────────────────────────────────────────

fn draw_header(ui: &mut egui::Ui, ctx: &egui::Context, admin_name: &str) -> bool {
    let mut wants_disconnect = false;

    let inner = Frame::none()
        .fill(SURFACE)
        .stroke(Stroke {
            width: 0.0,
            color: Color32::TRANSPARENT,
        })
        .inner_margin(Margin::symmetric(SPACE_MD, SPACE_MD - 2.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Pulsing live dot
                let t = ctx.input(|i| i.time);
                let phase = (t * (std::f64::consts::TAU / 1.6)).sin() as f32;
                let alpha = phase * 0.30 + 0.70;
                let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), Sense::hover());
                icons::pulse_dot(ui.painter(), rect.center(), 12.0, DANGER, alpha);

                ui.add_space(SPACE_XS + 2.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(
                        RichText::new("LIVE SESSION")
                            .size(9.5)
                            .color(TEXT_MUTED)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("Screen sharing in progress")
                            .size(11.5)
                            .color(TEXT_FAINT),
                    );
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Compact circular close button — ghost by default, danger on hover.
                    let (btn_rect, btn_resp) =
                        ui.allocate_exact_size(egui::vec2(28.0, 28.0), Sense::click());
                    let bg = if btn_resp.is_pointer_button_down_on() {
                        theme::DANGER_TINT
                    } else if btn_resp.hovered() {
                        theme::DANGER_TINT
                    } else {
                        Color32::TRANSPARENT
                    };
                    let icon_color = if btn_resp.hovered() {
                        DANGER
                    } else {
                        TEXT_MUTED
                    };
                    ui.painter().rect_filled(btn_rect, theme::r_md(), bg);
                    icons::close_x(ui.painter(), btn_rect.center(), 14.0, icon_color);
                    if btn_resp.clicked() {
                        wants_disconnect = true;
                    }

                    ui.add_space(SPACE_SM);

                    // Admin pill (avatar + name) — compact info chip
                    let (av_rect, _) =
                        ui.allocate_exact_size(egui::vec2(22.0, 22.0), Sense::hover());
                    icons::avatar(ui.painter(), av_rect.center(), 22.0, PRIMARY, admin_name);
                    ui.add_space(SPACE_XS + 2.0);
                    ui.label(
                        RichText::new(admin_name)
                            .size(12.0)
                            .color(TEXT_SUBTLE)
                            .strong(),
                    );
                });
            });
        });

    let header_rect = inner.response.rect;
    // Hairline divider beneath the header
    ui.painter().line_segment(
        [
            egui::pos2(header_rect.left(), header_rect.bottom()),
            egui::pos2(header_rect.right(), header_rect.bottom()),
        ],
        Stroke::new(1.0, BORDER),
    );

    // Drag handle covering the header area — `interact` won't conflict with
    // child widgets that already consumed events.
    let drag = ui.interact(header_rect, egui::Id::new("session_header_drag"), Sense::drag());
    if drag.drag_started() {
        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    wants_disconnect
}

// ── Empty state ────────────────────────────────────────────────────────────

fn draw_empty_state(ui: &mut egui::Ui) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(48.0, 48.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, theme::r_md(), PRIMARY_TINT);
        icons::eye(ui.painter(), rect.center(), 24.0, PRIMARY);
        ui.add_space(SPACE_MD);
        ui.label(
            RichText::new("No messages yet")
                .size(13.5)
                .color(TEXT_SUBTLE)
                .strong(),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new("Use chat to coordinate with your IT technician.")
                .size(12.0)
                .color(TEXT_MUTED),
        );
    });
}

// ── Chat bubble ────────────────────────────────────────────────────────────

fn draw_bubble(ui: &mut egui::Ui, line: &ChatLine, show_sender: bool, admin_name: &str) {
    let is_user = line.from.eq_ignore_ascii_case("user");
    let bg = if is_user { PRIMARY_TINT } else { SURFACE };
    let text_color = if is_user { PRIMARY_INK } else { TEXT };
    let border = if is_user {
        with_alpha(PRIMARY, 60)
    } else {
        BORDER
    };
    let label_color = if is_user {
        with_alpha(PRIMARY_INK, 180)
    } else {
        TEXT_MUTED
    };

    // Asymmetric corner: tighter on the speaker's side at the bottom for a
    // typical chat-tail feel without the cartoon point.
    let rounding = if is_user {
        Rounding {
            nw: theme::RADIUS_LG,
            ne: theme::RADIUS_LG,
            sw: theme::RADIUS_LG,
            se: 4.0,
        }
    } else {
        Rounding {
            nw: theme::RADIUS_LG,
            ne: theme::RADIUS_LG,
            sw: 4.0,
            se: theme::RADIUS_LG,
        }
    };

    let outer_layout = if is_user {
        Layout::right_to_left(Align::Min)
    } else {
        Layout::left_to_right(Align::Min)
    };

    ui.with_layout(outer_layout, |ui| {
        let max_w = (ui.available_width() * 0.78).max(150.0);

        Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.0, border))
            .rounding(rounding)
            .inner_margin(Margin::symmetric(SPACE_MD - 1.0, SPACE_SM))
            .show(ui, |ui| {
                ui.set_max_width(max_w);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);

                    if show_sender && !is_user {
                        let sender = if line.from.is_empty() {
                            admin_name
                        } else if line.from.eq_ignore_ascii_case("admin") {
                            admin_name
                        } else {
                            line.from.as_str()
                        };
                        ui.label(
                            RichText::new(sender)
                                .size(10.5)
                                .color(label_color)
                                .strong(),
                        );
                    }

                    ui.label(RichText::new(&line.text).size(13.0).color(text_color));

                    ui.label(
                        RichText::new(format_clock(line.ts))
                            .size(9.5)
                            .color(label_color),
                    );
                });
            });
    });
}

// ── Composer (input + send) ────────────────────────────────────────────────

struct ComposerResponse {
    field_response: egui::Response,
    send_clicked: bool,
}

fn composer(ui: &mut egui::Ui, buffer: &mut String, _focus_hint: bool) -> ComposerResponse {
    let height = 40.0;
    let total_w = ui.available_width();
    let send_w = 40.0;
    let gap = SPACE_SM;
    let field_w = (total_w - send_w - gap).max(80.0);
    let mut send_clicked = false;

    let field_response = ui.horizontal(|ui| {
        // ── Input field with restrained chrome ────────────────────────────
        // Use a subtle 1px border in all states (incl. focus) so the field
        // doesn't "shout" — color difference alone signals focus.
        let resp = ui.scope(|ui| {
            let widgets = &mut ui.style_mut().visuals.widgets;
            widgets.inactive.bg_fill      = SURFACE_ALT;
            widgets.inactive.weak_bg_fill = SURFACE_ALT;
            widgets.inactive.bg_stroke    = Stroke::new(1.0, BORDER);
            widgets.hovered.bg_fill       = SURFACE_ALT;
            widgets.hovered.weak_bg_fill  = SURFACE_ALT;
            widgets.hovered.bg_stroke     = Stroke::new(1.0, theme::BORDER_STRONG);
            widgets.active.bg_fill        = SURFACE;
            widgets.active.weak_bg_fill   = SURFACE;
            widgets.active.bg_stroke      = Stroke::new(1.0, PRIMARY);

            ui.add_sized(
                egui::vec2(field_w, height),
                egui::TextEdit::singleline(buffer)
                    .hint_text("Type a message…")
                    .margin(egui::vec2(SPACE_MD, SPACE_SM + 2.0))
                    .frame(true),
            )
        })
        .inner;

        ui.add_space(gap);

        // ── Send button ───────────────────────────────────────────────────
        // Disabled-looking when empty (greys out, no shadow) so the affordance
        // matches the "Enter to send" gesture.
        let (rect, btn) =
            ui.allocate_exact_size(egui::vec2(send_w, height), Sense::click());
        let has_text = !buffer.trim().is_empty();
        let bg = if !has_text {
            SURFACE_ALT
        } else if btn.is_pointer_button_down_on() {
            theme::PRIMARY_PRESS
        } else if btn.hovered() {
            theme::PRIMARY_HOVER
        } else {
            PRIMARY
        };
        let icon = if !has_text { TEXT_FAINT } else { Color32::WHITE };

        ui.painter().rect_filled(rect, theme::r_md(), bg);
        if !has_text {
            ui.painter()
                .rect_stroke(rect, theme::r_md(), Stroke::new(1.0, BORDER));
        }
        icons::send(ui.painter(), rect.center(), 15.0, icon);

        if btn.clicked() && has_text {
            send_clicked = true;
        }

        resp
    })
    .inner;

    ComposerResponse {
        field_response,
        send_clicked,
    }
}

// ── Status strip (footer below composer) ──────────────────────────────────

fn draw_status_strip(ui: &mut egui::Ui, elapsed: &Elapsed, chat: &ChatHandle) {
    ui.horizontal(|ui| {
        let (label, dot_color, text_color) = if chat.failed() {
            ("Disconnected", DANGER, DANGER)
        } else if !chat.connected() {
            ("Connecting…", theme::WARNING, TEXT_MUTED)
        } else {
            ("Connected", SUCCESS, TEXT_MUTED)
        };

        // Status dot: filled core + soft halo so it reads against any bg.
        let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), Sense::hover());
        ui.painter().circle_filled(
            rect.center(),
            5.0,
            theme::with_alpha(dot_color, 60),
        );
        ui.painter().circle_filled(rect.center(), 3.0, dot_color);
        ui.add_space(SPACE_XS + 2.0);
        ui.label(
            RichText::new(label)
                .size(11.0)
                .color(text_color)
                .strong(),
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(elapsed.format())
                    .size(11.0)
                    .color(TEXT)
                    .monospace()
                    .strong(),
            );
            ui.add_space(SPACE_XS + 2.0);
            ui.label(
                RichText::new("Duration")
                    .size(10.5)
                    .color(TEXT_FAINT),
            );
        });
    });
}

