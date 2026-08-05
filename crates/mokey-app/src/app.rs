use std::sync::mpsc::Receiver;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;

use egui::{Align2, Color32, FontId, Pos2, Rect as UiRect, Stroke, StrokeKind, ViewportCommand, ViewportId};
use mokey_backend::hotkey::HotkeyId;
use mokey_backend::mouse::MouseBackend;
use mokey_core::{Config, KeyEvent, Point};

use crate::controller::{Controller, ExecOutcome};

pub struct MokeyApp {
    controller: Controller,
    hotkeys: Receiver<HotkeyId>,
    global_keys: Receiver<KeyEvent>,
    capture: Arc<AtomicBool>,
    monitors: Vec<mokey_backend::platform::MonitorInfo>,
    settings_viewport: ViewportId,
    hud_visible: bool,
    settings_open: bool,
    settings_saved_hint: Option<String>,
}

impl MokeyApp {
    pub fn new(
        config: Config,
        mouse: Box<dyn MouseBackend>,
        hotkeys: Receiver<HotkeyId>,
        global_keys: Receiver<KeyEvent>,
        capture: Arc<AtomicBool>,
    ) -> MokeyApp {
        let monitors = mokey_backend::platform::list().unwrap_or_default();
        let controller = Controller::new(config, mouse);
        MokeyApp {
            controller,
            hotkeys,
            global_keys,
            capture,
            monitors,
            settings_viewport: ViewportId::from_hash_of("mokey-settings"),
            hud_visible: false,
            settings_open: false,
            settings_saved_hint: None,
        }
    }

    fn drain_hotkeys(&mut self, ctx: &egui::Context) {
        while let Ok(hk) = self.hotkeys.try_recv() {
            match hk {
                HotkeyId::Trigger => self.trigger(ctx),
                HotkeyId::Settings => self.settings_open = !self.settings_open,
            }
        }
    }

    fn drain_global_keys(&mut self, ctx: &egui::Context) {
        // Keys are only forwarded while drag capture is active.
        while self.capture.load(Ordering::Relaxed) {
            match self.global_keys.try_recv() {
                Ok(ev) => {
                    let outcome = self.controller.process(ev);
                    self.apply_outcome(ctx, &outcome);
                }
                Err(_) => break,
            }
        }
    }

    fn trigger(&mut self, ctx: &egui::Context) {
        if self.hud_visible {
            return;
        }
        let cursor = self.controller.mouse.location().ok();
        let monitor = match cursor {
            Some((x, y)) => self
                .monitors
                .iter()
                .find(|m| m.rect.contains(Point { x, y }))
                .or_else(|| self.monitors.first())
                .cloned(),
            None => self.monitors.first().cloned(),
        };
        let Some(monitor) = monitor else {
            return;
        };
        self.controller.start_session(monitor.rect);
        self.show_hud_over_monitor(ctx, &monitor);
    }

    fn show_hud_over_monitor(&mut self, ctx: &egui::Context, monitor: &mokey_backend::platform::MonitorInfo) {
        let pos = Pos2 {
            x: monitor.rect.x as f32 / monitor.scale as f32,
            y: monitor.rect.y as f32 / monitor.scale as f32,
        };
        let size = egui::vec2(
            monitor.rect.w as f32 / monitor.scale as f32,
            monitor.rect.h as f32 / monitor.scale as f32,
        );
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        self.hud_visible = true;
    }

    fn apply_outcome(&mut self, ctx: &egui::Context, outcome: &ExecOutcome) {
        if outcome.hide_hud {
            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        }
        let needs_delay = outcome.click.is_some() || outcome.press_drag.is_some();
        if needs_delay {
            // Let the OS process the window hide before injecting input.
            std::thread::sleep(Duration::from_millis(30));
            self.controller.apply_hidden_actions(outcome);
        }
        if outcome.capture_keys {
            self.capture.store(true, Ordering::Relaxed);
        }
        if outcome.show_hud {
            ctx.send_viewport_cmd(ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }
        if outcome.finished {
            self.hud_visible = false;
            self.capture.store(false, Ordering::Relaxed);
        }
    }

    fn handle_hud_keys(&mut self, ctx: &egui::Context) {
        if !self.hud_visible || self.controller.session.is_none() {
            return;
        }
        let events: Vec<egui::Event> = ctx.input(|i| i.events.clone());
        for ev in events {
            if !crate::keys::is_key_press(&ev) {
                continue;
            }
            if let egui::Event::Key { key, modifiers, .. } = ev {
                let mkey = crate::keys::map_key(key);
                if mkey == mokey_core::MokeyKey::Other {
                    continue;
                }
                let outcome = self.controller.process(KeyEvent {
                    key: mkey,
                    shift: modifiers.shift,
                });
                self.apply_outcome(ctx, &outcome);
            }
        }
    }

    fn draw_hud(&mut self, ctx: &egui::Context) {
        let Some(session) = self.controller.session.as_ref() else {
            return;
        };
        let opacity = self.controller.config.general.overlay_opacity();
        let bg = Color32::from_black_alpha((opacity * 255.0) as u8);
        let scale = ctx.pixels_per_point() as f64;
        let monitor = session.monitor;
        let origin = Pos2 {
            x: monitor.x as f32 / scale as f32,
            y: monitor.y as f32 / scale as f32,
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg))
            .show(ctx, |ui| {
                let painter = ui.painter();
                let region = session.region;
                let region_min = Pos2 {
                    x: (region.x as f64 / scale) as f32 - origin.x,
                    y: (region.y as f64 / scale) as f32 - origin.y,
                };
                let region_max = Pos2 {
                    x: ((region.x + region.w as i32) as f64 / scale) as f32 - origin.x,
                    y: ((region.y + region.h as i32) as f64 / scale) as f32 - origin.y,
                };

                painter.rect_stroke(
                    UiRect::from_min_max(region_min, region_max),
                    0.0,
                    Stroke::new(3.0_f32, Color32::from_rgb(80, 220, 255)),
                    StrokeKind::Inside,
                );

                let grid = session.grid();
                let label_font = FontId::proportional(26.0);
                for label in 1..=grid.label_count() {
                    if let Some(cell) = session.cell_rect(label) {
                        let min = Pos2 {
                            x: (cell.x as f64 / scale) as f32 - origin.x,
                            y: (cell.y as f64 / scale) as f32 - origin.y,
                        };
                        let max = Pos2 {
                            x: ((cell.x + cell.w as i32) as f64 / scale) as f32 - origin.x,
                            y: ((cell.y + cell.h as i32) as f64 / scale) as f32 - origin.y,
                        };
                        painter.rect_stroke(
                            UiRect::from_min_max(min, max),
                            0.0,
                            Stroke::new(1.5_f32, Color32::from_white_alpha(120)),
                            StrokeKind::Inside,
                        );
                        painter.text(
                            Pos2 {
                                x: (min.x + max.x) * 0.5,
                                y: (min.y + max.y) * 0.5,
                            },
                            Align2::CENTER_CENTER,
                            label.to_string(),
                            label_font.clone(),
                            Color32::from_white_alpha(220),
                        );
                    }
                }

                let vim = self.controller.config.vim.enabled;
                let hints = if vim {
                    "1-9 zoom · Enter click · m left · , mid · . right · hjkl move · e/y scroll · v drag · BS out · Esc cancel"
                } else {
                    "1-9 zoom · Enter click · Backspace zoom out · Esc cancel"
                };
                let width = ui.available_width();
                let bottom = ui.max_rect().bottom() - 4.0;
                let bar_rect = UiRect::from_min_max(
                    Pos2 { x: 0.0, y: bottom - 34.0 },
                    Pos2 { x: width, y: bottom },
                );
                painter.rect_filled(bar_rect, 0.0, Color32::from_black_alpha(140));
                painter.text(
                    Pos2 { x: width * 0.5, y: bottom - 17.0 },
                    Align2::CENTER_CENTER,
                    hints,
                    FontId::proportional(16.0),
                    Color32::WHITE,
                );

                let depth = session.depth;
                let status = format!(
                    "mokey · depth {}{}",
                    depth,
                    if vim { " · vim mode" } else { "" }
                );
                painter.text(
                    Pos2 { x: 12.0, y: 12.0 },
                    Align2::LEFT_TOP,
                    status,
                    FontId::proportional(15.0),
                    Color32::WHITE,
                );
            });
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let id = self.settings_viewport;
        ctx.show_viewport_immediate(
            id,
            egui::ViewportBuilder::default()
                .with_title("mokey settings")
                .with_inner_size([460.0, 560.0]),
            |ctx, class| {
                if class == egui::ViewportClass::Embedded {
                    return;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| self.settings_ui(ui));
                });
            },
        );
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        let mut cfg = self.controller.config.clone();

        ui.heading("mokey");
        ui.add_space(4.0);

        ui.separator();
        ui.label(egui::RichText::new("Vim mode (default OFF)").strong());
        ui.checkbox(&mut cfg.vim.enabled, "Enable vim-style control after grid zoom");
        if cfg.vim.enabled {
            ui.label("hjkl move · H/J/K/L fast move · m/,. click · e/y scroll · v drag");
        }

        ui.add_space(8.0);
        ui.separator();
        ui.label(egui::RichText::new("Grid").strong());
        ui.add(egui::Slider::new(&mut cfg.general.grid_size, 2..=5).text("grid size (n x n)"));
        ui.add(egui::Slider::new(&mut cfg.general.max_depth, 1..=6).text("max zoom depth"));
        ui.checkbox(&mut cfg.general.auto_click, "auto click at max depth");
        ui.add(
            egui::Slider::new(&mut cfg.general.overlay_bg_opacity, 0.0..=1.0)
                .text("overlay opacity"),
        );

        ui.add_space(8.0);
        ui.separator();
        ui.label(egui::RichText::new("Vim movement").strong());
        ui.add(egui::Slider::new(&mut cfg.general.move_step, 1..=100).text("hjkl step (px)"));
        ui.add(egui::Slider::new(&mut cfg.general.move_fast_step, 10..=500).text("HJKL step (px)"));

        ui.add_space(8.0);
        ui.separator();
        ui.label(egui::RichText::new("Hotkeys").strong());
        ui.label("Hotkey changes take effect after restart.");
        ui.horizontal(|ui| {
            ui.label("trigger");
            ui.add(egui::TextEdit::singleline(&mut cfg.general.trigger_hotkey).desired_width(180.0));
        });
        ui.horizontal(|ui| {
            ui.label("settings");
            ui.add(egui::TextEdit::singleline(&mut cfg.general.settings_hotkey).desired_width(180.0));
        });

        ui.add_space(12.0);
        if ui.button("Save").clicked() {
            self.controller.config = cfg.clone();
            let result = self.controller.config.save();
            match result {
                Ok(path) => self.settings_saved_hint = Some(format!("saved to {}", path.display())),
                Err(e) => self.settings_saved_hint = Some(format!("save failed: {e}")),
            }
        }
        if let Some(hint) = &self.settings_saved_hint {
            ui.label(hint);
        }
    }
}

impl eframe::App for MokeyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_hotkeys(ctx);
        self.drain_global_keys(ctx);
        self.handle_hud_keys(ctx);
        self.draw_hud(ctx);
        self.settings_window(ctx);
    }
}
