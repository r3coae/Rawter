use crate::icons::{self, Icon};
use crate::router::{destination_name, Classifier, Method, Optimize, CATALOG};
use crate::settings::{LaunchKey, Settings};
use crate::ui_main::kbd;
use crate::{ACCENT, DIM, INK, LINE, SWITCH_OFF, TEXT};
use egui::{
    pos2, vec2, Align, Button, ComboBox, CornerRadius, CursorIcon, FontId, Layout, Rect, Response,
    RichText, ScrollArea, Sense, Stroke, TextEdit, Ui, UiBuilder, Vec2,
};
use std::hash::Hash;

fn toggle(ui: &mut Ui, on: &mut bool) -> Response {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(30.0, 18.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    let t = ui.ctx().animate_bool_responsive(resp.id, *on);
    ui.painter().rect_filled(rect, CornerRadius::same(9), SWITCH_OFF.lerp_to_gamma(ACCENT, t));
    let x = egui::lerp((rect.left() + 9.0)..=(rect.right() - 9.0), t);
    let knob = if *on { INK } else { TEXT };
    ui.painter().circle_filled(pos2(x, rect.center().y), 6.0, knob);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

fn icon_button(ui: &mut Ui, icon: Icon, tip: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::click());
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand).on_hover_text(tip);
    let col = if resp.hovered() { TEXT } else { DIM };
    icons::image(icon, 16.0, col).paint_at(ui, rect);
    resp
}

fn hint(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(12.0).color(DIM));
}

fn section(ui: &mut Ui, first: bool, title: &str, hint_text: Option<&str>, add: impl FnOnce(&mut Ui)) {
    if !first {
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);
    }
    ui.label(RichText::new(title).size(13.0).strong().color(TEXT));
    if let Some(h) = hint_text {
        hint(ui, h);
    }
    ui.add_space(6.0);
    add(ui);
}

fn row(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(vec2(170.0, 24.0), Layout::left_to_right(Align::Center), |ui| {
            ui.label(RichText::new(label).color(TEXT));
        });
        add(ui);
    });
}

fn combo<T: PartialEq + Copy>(ui: &mut Ui, salt: impl Hash + std::fmt::Debug, width: f32, value: &mut T, options: &[(T, &str)]) {
    let current = options.iter().find(|(v, _)| v == value).map_or("", |(_, l)| *l);
    ComboBox::from_id_salt(salt).width(width).selected_text(current).show_ui(ui, |ui| {
        for (v, l) in options {
            ui.selectable_value(value, *v, *l);
        }
    });
}

fn dest_combo(ui: &mut Ui, salt: impl Hash + std::fmt::Debug, width: f32, value: &mut String) {
    ComboBox::from_id_salt(salt).width(width).selected_text(destination_name(value)).show_ui(ui, |ui| {
        for d in CATALOG {
            ui.selectable_value(value, d.id.to_string(), d.name);
        }
    });
}

/// Returns true when the user clicked the close icon. Esc and Ctrl+, are handled in main.rs.
pub fn show(ui: &mut Ui, s: &mut Settings) -> bool {
    let rect = ui.max_rect();
    let mut close = false;

    // Header, 48px
    let header = Rect::from_min_size(rect.min, vec2(rect.width(), 48.0));
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(Rect::from_min_max(
                pos2(header.left() + 24.0, header.top()),
                pos2(header.right() - 24.0, header.bottom()),
            ))
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.label(RichText::new("Settings").size(15.0).strong().color(TEXT));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_button(ui, Icon::Close, "Close").clicked() {
                    close = true;
                }
                ui.add_space(4.0);
                kbd(ui, "Esc");
            });
        },
    );
    ui.painter().hline(rect.x_range(), header.bottom(), Stroke::new(1.0, LINE));

    // Body
    let body = Rect::from_min_max(
        pos2(rect.left() + 24.0, header.bottom() + 8.0),
        pos2(rect.right() - 24.0, rect.bottom() - 8.0),
    );
    ui.scope_builder(UiBuilder::new().max_rect(body), |ui| {
        ScrollArea::vertical().id_salt("settings_scroll").auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.add_space(8.0);

            section(
                ui,
                true,
                "Destinations",
                Some("Tools tasks can be sent to. Use {task} where the task text goes."),
                |ui| {
                    for d in s.destinations.iter_mut() {
                        ui.horizontal(|ui| {
                            toggle(ui, &mut d.enabled);
                            let name_col = if d.enabled { TEXT } else { DIM };
                            ui.allocate_ui_with_layout(
                                vec2(112.0, 22.0),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    ui.label(RichText::new(destination_name(&d.id)).color(name_col));
                                },
                            );
                            combo(
                                ui,
                                ("method", d.id.clone()),
                                68.0,
                                &mut d.method,
                                &[(Method::Url, "URL"), (Method::Cli, "CLI")],
                            );
                            ui.add(
                                TextEdit::singleline(&mut d.value)
                                    .font(FontId::monospace(12.0))
                                    .desired_width(f32::INFINITY),
                            );
                        });
                    }
                },
            );

            section(ui, false, "Routing", None, |ui| {
                row(ui, "Routing mode", |ui| {
                    combo(
                        ui,
                        "classifier",
                        180.0,
                        &mut s.classifier,
                        &[(Classifier::Auto, "Auto (scored)"), (Classifier::Rules, "Keyword rules only")],
                    );
                });
                row(ui, "Optimize for", |ui| {
                    combo(
                        ui,
                        "optimize",
                        180.0,
                        &mut s.optimize,
                        &[(Optimize::Quality, "Quality"), (Optimize::Speed, "Speed"), (Optimize::Cost, "Cost")],
                    );
                });
                row(ui, "Confirm before launch", |ui| {
                    toggle(ui, &mut s.confirm);
                });
                hint(ui, "Shows the chosen destination first. Enter confirms, Tab shows the next best.");
                row(ui, "Fallback destination", |ui| {
                    dest_combo(ui, "fallback", 180.0, &mut s.fallback);
                });

                ui.add_space(4.0);
                ui.label(RichText::new("Override rules").color(TEXT));
                hint(ui, "Override rules run before the scorer, top to bottom. Separate keywords with commas.");
                let mut remove: Option<usize> = None;
                for (i, r) in s.rules.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add(
                            TextEdit::singleline(&mut r.keywords)
                                .hint_text(RichText::new("keywords, like, these").color(DIM))
                                .desired_width(260.0),
                        );
                        dest_combo(ui, ("rule_target", r.id), 140.0, &mut r.target);
                        if icon_button(ui, Icon::Trash, "Delete rule").clicked() {
                            remove = Some(i);
                        }
                    });
                }
                if let Some(i) = remove {
                    s.rules.remove(i);
                }
                let add = Button::new(RichText::new("+ Add rule").color(ACCENT)).frame(false);
                if ui.add(add).on_hover_cursor(CursorIcon::PointingHand).clicked() {
                    s.add_rule();
                }
            });

            section(ui, false, "Launch", None, |ui| {
                row(ui, "Launch key", |ui| {
                    combo(
                        ui,
                        "launch_key",
                        140.0,
                        &mut s.launch_key,
                        &[(LaunchKey::CtrlEnter, "Ctrl + Enter"), (LaunchKey::Enter, "Enter")],
                    );
                });
                row(ui, "Clear input after launch", |ui| {
                    toggle(ui, &mut s.clear_after);
                });
            });

            section(ui, false, "History", None, |ui| {
                row(ui, "Keep task history", |ui| {
                    toggle(ui, &mut s.keep_history);
                });
                let n = s.history.len();
                ui.horizontal(|ui| {
                    let noun = if n == 1 { "saved task" } else { "saved tasks" };
                    ui.label(RichText::new(format!("{n} {noun}")).color(DIM));
                    if ui.add_enabled(n > 0, Button::new("Clear history")).clicked() {
                        s.history.clear();
                    }
                });
            });

            ui.add_space(16.0);
        });
    });

    close
}
