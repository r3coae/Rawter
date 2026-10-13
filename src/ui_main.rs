use crate::icons::{self, Icon};
use crate::launch;
use crate::router::{destination_name, route_task};
use crate::settings::{LaunchKey, Settings};
use crate::{ACCENT, DIM, FOCUS, INK, LINE, TEXT, WELL};
use egui::{
    pos2, vec2, Button, Color32, Context, CornerRadius, CursorIcon, Frame, Id, Key, Margin, Rect,
    RichText, ScrollArea, Sense, Stroke, TextEdit, Ui, UiBuilder, Vec2,
};

const PLACEHOLDER: &str = "What do you need done? (e.g., Build a React login screen...)";
const ROUTING_SECS: f64 = 0.7;

pub enum DoneKind {
    Launched,
    Copied,
    Command,
}

pub enum Status {
    Idle,
    EmptyTask,
    NoDest,
    Confirm { task: String, ranked: Vec<String>, idx: usize, labels: String },
    Routing { task: String, id: String, labels: String, start: f64 },
    Done { id: String, kind: DoneKind },
    Error,
}

pub struct MainState {
    pub task: String,
    pub status: Status,
    /// request focus on the text area next frame
    pub focus: bool,
}

impl Default for MainState {
    fn default() -> Self {
        Self { task: String::new(), status: Status::Idle, focus: true }
    }
}

pub fn take_key(ctx: &Context, key: Key, ok: impl Fn(&egui::Modifiers) -> bool) -> bool {
    ctx.input_mut(|i| {
        let mut hit = false;
        i.events.retain(|e| match e {
            egui::Event::Key { key: k, pressed: true, modifiers, .. } if *k == key && ok(modifiers) => {
                hit = true;
                false
            }
            _ => true,
        });
        hit
    })
}

pub fn kbd(ui: &mut Ui, text: &str) {
    Frame::new()
        .fill(WELL)
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(CornerRadius::same(3))
        .inner_margin(Margin::symmetric(5, 1))
        .show(ui, |ui| {
            ui.label(RichText::new(text).monospace().size(11.0).color(DIM));
        });
}

impl MainState {
    /// Returns true when the user asked to open settings.
    pub fn show(&mut self, ui: &mut Ui, settings: &mut Settings) -> bool {
        let ctx = ui.ctx().clone();
        self.handle_keys(&ctx, settings);
        self.tick(&ctx, settings);

        let rect = ui.max_rect();
        let mut open_settings = false;

        // Gear, floating top-right
        let gear = Rect::from_min_size(pos2(rect.right() - 28.0, rect.top() + 12.0), Vec2::splat(16.0));
        let gresp = ui
            .interact(gear.expand(4.0), Id::new("gear"), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text("Settings (Ctrl + ,)");
        let gcol = if gresp.hovered() { TEXT } else { DIM };
        icons::image(Icon::Settings, 16.0, gcol).paint_at(ui, gear);
        if gresp.clicked() {
            open_settings = true;
        }

        // Text area
        let footer_top = rect.bottom() - 90.0;
        let text_rect = Rect::from_min_max(
            pos2(rect.left() + 24.0, rect.top() + 48.0),
            pos2(rect.right() - 24.0, footer_top - 16.0),
        );
        let input_id = Id::new("task_input");
        let focused = ctx.memory(|m| m.has_focus(input_id));
        let inner_h = text_rect.height() - 22.0;
        let rows = ((inner_h / 18.0).floor() as usize).max(1);

        let resp = ui
            .scope_builder(UiBuilder::new().max_rect(text_rect), |ui| {
                Frame::new()
                    .fill(WELL)
                    .stroke(Stroke::new(1.0, if focused { FOCUS } else { LINE }))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(10))
                    .show(ui, |ui| {
                        ui.set_min_width(text_rect.width() - 22.0);
                        ScrollArea::vertical()
                            .id_salt("task_scroll")
                            .max_height(inner_h)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.add(
                                    TextEdit::multiline(&mut self.task)
                                        .id(input_id)
                                        .frame(Frame::NONE)
                                        .font(egui::FontId::monospace(15.0))
                                        .hint_text(RichText::new(PLACEHOLDER).color(DIM))
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(rows),
                                )
                            })
                            .inner
                    })
                    .inner
            })
            .inner;

        if self.focus || matches!(self.status, Status::Confirm { .. }) {
            resp.request_focus();
            self.focus = false;
        }
        if resp.changed()
            && matches!(self.status, Status::Confirm { .. } | Status::EmptyTask | Status::NoDest | Status::Error)
        {
            self.status = Status::Idle;
        }

        // Footer
        let footer_rect =
            Rect::from_min_max(pos2(rect.left() + 24.0, footer_top), pos2(rect.right() - 24.0, rect.bottom()));
        let mut clicked = false;
        ui.scope_builder(UiBuilder::new().max_rect(footer_rect), |ui| {
            ui.horizontal(|ui| {
                let btn = Button::image_and_text(
                    icons::image(Icon::Route, 16.0, INK),
                    RichText::new("Route & Launch").size(13.0).strong().color(INK),
                )
                .fill(ACCENT)
                .corner_radius(CornerRadius::same(8))
                .min_size(vec2(150.0, 40.0));
                if ui.add(btn).on_hover_cursor(CursorIcon::PointingHand).clicked() {
                    clicked = true;
                }
                ui.add_space(16.0);
                self.status_ui(ui);
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let dim = |s: &str| RichText::new(s).size(12.0).color(DIM);
                ui.label(dim("Press"));
                match settings.launch_key {
                    LaunchKey::CtrlEnter => {
                        kbd(ui, "Ctrl + Enter");
                        ui.label(dim("to launch"));
                    }
                    LaunchKey::Enter => {
                        kbd(ui, "Enter");
                        ui.label(dim("to launch (Shift + Enter for a new line)"));
                    }
                }
            });
        });
        if clicked {
            self.activate(&ctx, settings);
        }

        open_settings
    }

    fn status_ui(&self, ui: &mut Ui) {
        let mono = |s: String, c: Color32| RichText::new(s).monospace().size(13.0).color(c);
        match &self.status {
            Status::Idle => {}
            Status::EmptyTask => {
                ui.label(mono("Describe a task first.".into(), DIM));
            }
            Status::NoDest => {
                ui.label(mono("No destination enabled. Check Settings.".into(), DIM));
            }
            Status::Confirm { ranked, idx, .. } => {
                ui.label(mono(format!("Route to {}?", destination_name(&ranked[*idx])), TEXT));
                ui.label(mono("Enter to confirm, Tab for next best".into(), DIM));
            }
            Status::Routing { id, labels, .. } => {
                let angle = (ui.input(|i| i.time) * 6.0) as f32;
                ui.add(icons::image(Icon::Loader, 14.0, ACCENT).rotate(angle, Vec2::splat(0.5)));
                ui.label(mono(format!("Routing to {}...", destination_name(id)), TEXT));
                if !labels.is_empty() {
                    ui.label(mono(labels.clone(), DIM));
                }
            }
            Status::Done { id, kind } => {
                let name = destination_name(id);
                let msg = match kind {
                    DoneKind::Launched => format!("Launched in {name}"),
                    DoneKind::Copied => format!("Opened {name}, task copied"),
                    DoneKind::Command => format!("Command copied for {name}"),
                };
                ui.add(icons::image(Icon::Check, 14.0, ACCENT));
                ui.label(mono(msg, TEXT));
            }
            Status::Error => {
                ui.label(mono("Couldn't open the browser.".into(), DIM));
            }
        }
    }

    fn handle_keys(&mut self, ctx: &Context, settings: &mut Settings) {
        if matches!(self.status, Status::Confirm { .. }) {
            if take_key(ctx, Key::Escape, |_| true) {
                self.status = Status::Idle;
            } else if take_key(ctx, Key::Tab, |m| !m.shift) {
                if let Status::Confirm { ranked, idx, .. } = &mut self.status {
                    *idx = (*idx + 1) % ranked.len();
                }
            } else if take_key(ctx, Key::Enter, |_| true) {
                self.confirm(ctx);
            }
            return;
        }
        let by_ctrl = take_key(ctx, Key::Enter, |m| m.command);
        let by_plain = settings.launch_key == LaunchKey::Enter
            && take_key(ctx, Key::Enter, |m| !m.command && !m.shift && !m.alt);
        if by_ctrl || by_plain {
            self.activate(ctx, settings);
        }
    }

    /// Validate, route, then confirm or start routing.
    fn activate(&mut self, ctx: &Context, settings: &Settings) {
        match self.status {
            Status::Routing { .. } => return,
            Status::Confirm { .. } => return self.confirm(ctx),
            _ => {}
        }
        let task = self.task.trim().to_string();
        if task.is_empty() {
            self.status = Status::EmptyTask;
            return;
        }
        if !settings.any_enabled() {
            self.status = Status::NoDest;
            return;
        }
        let r = route_task(&task, &settings.route_cfg());
        let Some(id) = r.id.clone() else {
            self.status = Status::NoDest;
            return;
        };
        let labels = r.labels.join(" + ");
        if settings.confirm {
            self.status = Status::Confirm { task, ranked: r.ranked, idx: 0, labels };
        } else {
            self.begin_routing(ctx, task, id, labels);
        }
    }

    fn confirm(&mut self, ctx: &Context) {
        if let Status::Confirm { task, ranked, idx, labels } = &self.status {
            let (task, id) = (task.clone(), ranked[*idx].clone());
            let labels = if *idx == 0 { labels.clone() } else { String::new() };
            self.begin_routing(ctx, task, id, labels);
        }
    }

    fn begin_routing(&mut self, ctx: &Context, task: String, id: String, labels: String) {
        let start = ctx.input(|i| i.time);
        self.status = Status::Routing { task, id, labels, start };
    }

    fn tick(&mut self, ctx: &Context, settings: &mut Settings) {
        let done = match &self.status {
            Status::Routing { id, task, start, .. } => {
                if ctx.input(|i| i.time) - start >= ROUTING_SECS {
                    Some((id.clone(), task.clone()))
                } else {
                    ctx.request_repaint();
                    None
                }
            }
            _ => None,
        };
        if let Some((id, task)) = done {
            self.finish(ctx, &id, &task, settings);
        }
    }

    fn finish(&mut self, ctx: &Context, id: &str, task: &str, settings: &mut Settings) {
        let Some(d) = settings.dest(id) else {
            self.status = Status::Error;
            return;
        };
        match launch::run(ctx, d, task) {
            Ok(kind) => {
                self.status = Status::Done { id: id.to_string(), kind };
                if settings.clear_after {
                    self.task.clear();
                }
                settings.push_history(task, id);
                let _ = settings.save();
            }
            Err(_) => self.status = Status::Error,
        }
    }
}
