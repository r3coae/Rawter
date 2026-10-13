mod icons;
mod launch;
mod router;
mod settings;
mod ui_main;
mod ui_settings;

use egui::{Color32, CornerRadius, FontId, Frame, Key, Modifiers, Shadow, Stroke, TextStyle};
use settings::Settings;

pub const WINDOW: Color32 = Color32::from_rgb(0x14, 0x14, 0x16);
pub const WELL: Color32 = Color32::from_rgb(0x19, 0x19, 0x1c);
pub const RAISED: Color32 = Color32::from_rgb(0x20, 0x20, 0x24);
pub const LINE: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2f);
pub const TEXT: Color32 = Color32::from_rgb(0xd8, 0xd6, 0xd0);
pub const DIM: Color32 = Color32::from_rgb(0x6f, 0x6e, 0x72);
pub const ACCENT: Color32 = Color32::from_rgb(0x9f, 0xb4, 0xd6);
pub const INK: Color32 = Color32::from_rgb(0x10, 0x12, 0x16);
pub const FOCUS: Color32 = Color32::from_rgb(0x3d, 0x46, 0x58);
pub const SWITCH_OFF: Color32 = Color32::from_rgb(0x34, 0x34, 0x3a);

fn apply_theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = WINDOW;
    v.window_fill = WINDOW;
    v.extreme_bg_color = WELL;
    v.faint_bg_color = WELL;
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.window_stroke = Stroke::new(1.0, LINE);
    v.selection.bg_fill = ACCENT.gamma_multiply(0.35);
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.text_cursor.stroke = Stroke::new(2.0, ACCENT);

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = WINDOW;
    w.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
    w.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    w.inactive.bg_fill = WELL;
    w.inactive.weak_bg_fill = WELL;
    w.inactive.bg_stroke = Stroke::new(1.0, LINE);
    w.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    w.hovered.bg_fill = RAISED;
    w.hovered.weak_bg_fill = RAISED;
    w.hovered.bg_stroke = Stroke::new(1.0, LINE);
    w.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    w.active.bg_fill = RAISED;
    w.active.weak_bg_fill = RAISED;
    w.active.bg_stroke = Stroke::new(1.0, FOCUS);
    w.active.fg_stroke = Stroke::new(1.0, TEXT);
    w.open.bg_fill = RAISED;
    w.open.weak_bg_fill = RAISED;
    for s in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
        s.corner_radius = CornerRadius::same(6);
        s.expansion = 0.0;
    }
    ctx.set_visuals(v);

    ctx.global_style_mut(|s| {
        s.text_styles.insert(TextStyle::Body, FontId::proportional(13.0));
        s.text_styles.insert(TextStyle::Button, FontId::proportional(13.0));
        s.text_styles.insert(TextStyle::Small, FontId::proportional(11.0));
        s.text_styles.insert(TextStyle::Monospace, FontId::monospace(13.0));
    });
}

#[derive(PartialEq)]
enum Screen {
    Main,
    Settings,
}

struct App {
    settings: Settings,
    screen: Screen,
    main: ui_main::MainState,
}

impl App {
    fn new() -> Self {
        Self { settings: Settings::load(), screen: Screen::Main, main: ui_main::MainState::default() }
    }

    fn toggle_settings(&mut self) {
        if self.screen == Screen::Main {
            self.screen = Screen::Settings;
        } else {
            let _ = self.settings.save();
            self.screen = Screen::Main;
            self.main.focus = true;
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Comma)) {
            self.toggle_settings();
        }
        if self.screen == Screen::Settings && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.toggle_settings();
        }

        egui::CentralPanel::default().frame(Frame::new().fill(WINDOW)).show(ui, |ui| match self.screen {
            Screen::Main => {
                if self.main.show(ui, &mut self.settings) {
                    self.toggle_settings();
                }
            }
            Screen::Settings => {
                let before = self.settings.clone();
                let close = ui_settings::show(ui, &mut self.settings);
                if self.settings != before {
                    let _ = self.settings.save();
                }
                if close {
                    self.toggle_settings();
                }
            }
        });
    }
}

fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/rawter-256.png"))
        .expect("valid icon");
    let size = [700.0, 450.0];
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_max_inner_size(size)
            .with_resizable(false)
            .with_title("rawter")
            .with_app_id("rawter")
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "rawter",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(App::new()))
        }),
    )
}