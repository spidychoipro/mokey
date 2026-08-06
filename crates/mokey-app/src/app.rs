use std::sync::mpsc::Receiver;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};

use egui::{
    Align2, Color32, FontId, Pos2, Rect as UiRect, Stroke, StrokeKind, TextureHandle,
    TextureOptions, ViewportCommand, ViewportId,
};
use mokey_backend::hotkey::HotkeyId;
use mokey_backend::mouse::{MouseBackend, MouseButton};
use mokey_core::{Config, KeyEvent, MokeyKey, Point, Rgba, Theme};

use crate::controller::{Controller, ExecOutcome};

#[allow(unused)]
fn dbg(msg: impl AsRef<str>) {
    if std::env::var_os("MOKEY_DEBUG").is_none() {
        return;
    }
    use std::io::Write;
    let desk = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into());
    let path = std::path::Path::new(&desk).join("Desktop").join("mokey-debug.log");
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "[{ts}] {}", msg.as_ref());
    }
}

fn egui_key_to_mokey(key: egui::Key) -> Option<MokeyKey> {
    use egui::Key as K;
    let mk = match key {
        K::Num1 => MokeyKey::Digit(1),
        K::Num2 => MokeyKey::Digit(2),
        K::Num3 => MokeyKey::Digit(3),
        K::Num4 => MokeyKey::Digit(4),
        K::Num5 => MokeyKey::Digit(5),
        K::Num6 => MokeyKey::Digit(6),
        K::Num7 => MokeyKey::Digit(7),
        K::Num8 => MokeyKey::Digit(8),
        K::Num9 => MokeyKey::Digit(9),
        K::Escape => MokeyKey::Escape,
        K::Enter => MokeyKey::Enter,
        K::Space => MokeyKey::Space,
        K::Backspace => MokeyKey::Backspace,
        K::H => MokeyKey::H,
        K::J => MokeyKey::J,
        K::K => MokeyKey::K,
        K::L => MokeyKey::L,
        K::M => MokeyKey::M,
        K::E => MokeyKey::E,
        K::Y => MokeyKey::Y,
        K::V => MokeyKey::V,
        K::Comma => MokeyKey::Comma,
        K::Period => MokeyKey::Period,
        _ => return None,
    };
    Some(mk)
}

fn to_color32(c: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

/// A pointer action that must run only after the HUD window has actually
/// moved offscreen (viewport commands are applied at the end of the frame).
#[derive(Clone, Copy)]
struct PendingAction {
    click: Option<MouseButton>,
    press_drag: Option<bool>,
    at: Instant,
}

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
    settings_pos: Option<Pos2>,
    settings_cfg: Option<Config>,
    theme_editor_open: bool,
    theme_draft_name: String,
    theme_draft: Theme,
    applied_theme: Option<String>,
    hud_bg: Option<TextureHandle>,
    hud_offscreen: bool,
    hud_pos: Option<Pos2>,
    focus_log_at: Option<Instant>,
    pending: Option<PendingAction>,
}

impl MokeyApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        config: Config,
        mouse: Box<dyn MouseBackend>,
    ) -> MokeyApp {
        let ctx = cc.egui_ctx.clone();
        let wake_ctx = ctx.clone();
        let wake: Option<Arc<dyn Fn() + Send + Sync>> = Some(Arc::new(move || {
            wake_ctx.request_repaint();
        }));
        let input = mokey_backend::hotkey::spawn(
            &config.general.trigger_hotkey,
            &config.general.settings_hotkey,
            wake,
        )
        .expect("failed to register hotkeys");
        let capture = input.capture.clone();
        let monitors = mokey_backend::platform::list().unwrap_or_default();
        let controller = Controller::new(config, mouse);
        MokeyApp {
            controller,
            hotkeys: input.hotkeys,
            global_keys: input.keys,
            capture,
            monitors,
            settings_viewport: ViewportId::from_hash_of("mokey-settings"),
            hud_visible: false,
            settings_open: false,
            settings_saved_hint: None,
            settings_pos: None,
            settings_cfg: None,
            theme_editor_open: false,
            theme_draft_name: String::new(),
            theme_draft: Theme::dark(),
            applied_theme: None,
            hud_bg: None,
            hud_offscreen: true,
            hud_pos: None,
            focus_log_at: None,
            pending: None,
        }
    }

    fn flush_pending(&mut self) {
        let Some(p) = self.pending else {
            return;
        };
        if p.at.elapsed() < Duration::from_millis(60) {
            return;
        }
        self.pending = None;
        let loc = self.controller.mouse.location().ok();
        dbg(format!(
            "APP inject click={:?} press_drag={:?} at loc={loc:?}",
            p.click, p.press_drag
        ));
        let out = ExecOutcome {
            click: p.click,
            press_drag: p.press_drag,
            ..Default::default()
        };
        self.controller.apply_hidden_actions(&out);
    }

    fn move_hud_offscreen(&mut self, ctx: &egui::Context) {
        if !self.hud_offscreen {
            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::new(-32000.0, -32000.0)));
            self.hud_offscreen = true;
        }
    }

    fn move_hud_on_screen(&mut self, ctx: &egui::Context, pos: Pos2) {
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        self.hud_offscreen = false;
        self.hud_pos = Some(pos);
    }

    fn drain_hotkeys(&mut self, ctx: &egui::Context) {
        while let Ok(hk) = self.hotkeys.try_recv() {
            match hk {
                HotkeyId::Trigger => self.trigger(ctx),
                HotkeyId::Settings => {
                    if !self.settings_open {
                        self.settings_pos = self.default_settings_pos();
                        self.settings_cfg = Some(self.controller.config.clone());
                    }
                    self.settings_open = !self.settings_open;
                }
            }
        }
    }

    fn default_settings_pos(&self) -> Option<Pos2> {
        let cursor = self.controller.mouse.location().ok();
        let monitor = match cursor {
            Some((x, y)) => self
                .monitors
                .iter()
                .find(|m| m.rect.contains(Point { x, y }))
                .or_else(|| self.monitors.first())
                .cloned(),
            None => self.monitors.first().cloned(),
        }?;
        let cx = (monitor.rect.x as f32 + monitor.rect.w as f32 / 2.0) / monitor.scale as f32;
        let cy = (monitor.rect.y as f32 + monitor.rect.h as f32 / 2.0) / monitor.scale as f32;
        Some(Pos2::new(cx - 230.0, cy - 280.0))
    }

    fn handle_hud_input(&mut self, ctx: &egui::Context) {
        if !self.hud_visible {
            return;
        }
        let mut keys: Vec<KeyEvent> = Vec::new();
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, physical_key, pressed, repeat, modifiers, .. } = event {
                    if *pressed && !*repeat {
                        let mapped = physical_key
                            .and_then(egui_key_to_mokey)
                            .or_else(|| egui_key_to_mokey(*key));
                        dbg(format!(
                            "EGUI raw: key={key:?} phys={physical_key:?} shift={} mapped={mapped:?}",
                            modifiers.shift
                        ));
                        if let Some(mk) = mapped {
                            keys.push(KeyEvent { key: mk, shift: modifiers.shift });
                        }
                    }
                }
            }
        });
        for ev in keys {
            let outcome = self.controller.process(ev);
            dbg(format!(
                "EGUI key {ev:?} -> hide={} show={} click={:?} finished={}",
                outcome.hide_hud,
                outcome.show_hud,
                outcome.click,
                outcome.finished
            ));
            self.apply_outcome(ctx, &outcome);
        }
    }

    fn drain_global_keys(&mut self, ctx: &egui::Context) {
        // Keys are only forwarded while drag capture is active.
        while self.capture.load(Ordering::Relaxed) {
            match self.global_keys.try_recv() {
                Ok(ev) => {
                    let outcome = self.controller.process(ev);
                    dbg(format!(
                        "APP recv {ev:?} -> hide={} show={} click={:?} finished={}",
                        outcome.hide_hud,
                        outcome.show_hud,
                        outcome.click,
                        outcome.finished
                    ));
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
        self.hud_bg = capture_monitor_bg(ctx, &monitor);
        self.controller.start_session(monitor.rect);
        self.show_hud_over_monitor(ctx, &monitor);
        dbg("APP trigger: hud shown");
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
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        self.hud_visible = true;
        self.move_hud_on_screen(ctx, pos);
    }

    fn apply_outcome(&mut self, ctx: &egui::Context, outcome: &ExecOutcome) {
        if outcome.hide_hud {
            self.move_hud_offscreen(ctx);
            self.hud_bg = None;
        }
        if outcome.capture_keys {
            self.capture.store(true, Ordering::Relaxed);
        }
        if outcome.show_hud {
            let pos = self.hud_pos.unwrap_or(Pos2::ZERO);
            self.move_hud_on_screen(ctx, pos);
        }
        if outcome.finished {
            self.hud_visible = false;
            self.capture.store(false, Ordering::Relaxed);
        }
        if outcome.click.is_some() || outcome.press_drag.is_some() {
            if outcome.hide_hud {
                // Window move commands apply at the end of this frame, so the
                // HUD still covers the screen here. Inject on a later frame.
                self.pending = Some(PendingAction {
                    click: outcome.click,
                    press_drag: outcome.press_drag,
                    at: Instant::now(),
                });
                ctx.request_repaint();
            } else {
                // e.g. drag release while the HUD returns: window is already
                // offscreen, so inject immediately.
                self.controller.apply_hidden_actions(outcome);
            }
        }
    }

    fn draw_hud(&mut self, ctx: &egui::Context) {
        let Some(session) = self.controller.session.as_ref() else {
            return;
        };
        let opacity = self.controller.config.general.overlay_opacity();
        let theme = Theme::resolve(
            &self.controller.config.general.theme,
            &self.controller.config.custom_themes,
        );
        let bg = Color32::from_rgba_unmultiplied(
            theme.overlay.r,
            theme.overlay.g,
            theme.overlay.b,
            (opacity * 255.0) as u8,
        );
        let scale = ctx.pixels_per_point() as f64;
        let monitor = session.monitor;
        let origin = Pos2 {
            x: monitor.x as f32 / scale as f32,
            y: monitor.y as f32 / scale as f32,
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::BLACK))
            .show(ctx, |ui| {
                let painter = ui.painter();
                let full = ui.max_rect();
                if let Some(tex) = &self.hud_bg {
                    painter.image(
                        tex.id(),
                        full,
                        UiRect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                painter.rect_filled(full, 0.0, bg);

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
                    Stroke::new(3.0_f32, to_color32(theme.accent)),
                    StrokeKind::Inside,
                );

                let grid = session.grid();
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
                        let cell_min = (max.x - min.x).min(max.y - min.y);
                        let stroke = (cell_min / 12.0).clamp(0.5, 1.5);
                        painter.rect_stroke(
                            UiRect::from_min_max(min, max),
                            0.0,
                            Stroke::new(stroke, to_color32(theme.grid)),
                            StrokeKind::Inside,
                        );
                        if cell_min >= 20.0 {
                            let font_size = (cell_min * 0.5).clamp(12.0, 26.0);
                            painter.text(
                                Pos2 {
                                    x: (min.x + max.x) * 0.5,
                                    y: (min.y + max.y) * 0.5,
                                },
                                Align2::CENTER_CENTER,
                                label.to_string(),
                                FontId::proportional(font_size),
                                to_color32(theme.label),
                            );
                        }
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
                painter.rect_filled(bar_rect, 0.0, to_color32(theme.hint_bg));
                painter.text(
                    Pos2 { x: width * 0.5, y: bottom - 17.0 },
                    Align2::CENTER_CENTER,
                    hints,
                    FontId::proportional(16.0),
                    to_color32(theme.hint_text),
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
                    to_color32(theme.status),
                );
            });
    }

    fn apply_visuals(&mut self, ctx: &egui::Context) {
        let theme = Theme::resolve(
            &self.controller.config.general.theme,
            &self.controller.config.custom_themes,
        );
        let key = format!("{}{}", theme.name, if theme.dark { ":d" } else { ":l" });
        if self.applied_theme.as_deref() == Some(key.as_str()) {
            return;
        }
        let mut visuals = if theme.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = to_color32(theme.panel);
        visuals.window_fill = to_color32(theme.bg);
        visuals.override_text_color = Some(to_color32(theme.text));
        ctx.set_visuals(visuals);
        self.applied_theme = Some(key);
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let id = self.settings_viewport;
        let builder = egui::ViewportBuilder::default()
            .with_title("mokey settings")
            .with_inner_size([460.0, 560.0]);
        ctx.show_viewport_immediate(id, builder, |ctx, class| {
            if class == egui::ViewportClass::Embedded {
                return;
            }
            if ctx.input(|i| i.viewport().close_requested()) {
                self.settings_open = false;
            }
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.settings_ui(ui));
            });
        });
        if let Some(pos) = self.settings_pos.take() {
            // The viewport exists now; command it onto a visible monitor. The
            // builder's position alone can't fight the offscreen parent.
            ctx.send_viewport_cmd_to(id, ViewportCommand::OuterPosition(pos));
            ctx.send_viewport_cmd_to(id, ViewportCommand::Focus);
        }
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        if self.settings_cfg.is_none() {
            self.settings_cfg = Some(self.controller.config.clone());
        }
        let cfg = self.settings_cfg.as_mut().expect("settings_cfg set above");

        ui.heading("mokey");
        ui.label(egui::RichText::new("Keyboard mouse control").weak());
        ui.add_space(8.0);

        // ---- Theme ----
        ui.separator();
        ui.label(egui::RichText::new("Theme").strong());
        let theme_name = cfg.general.theme.clone();
        let custom_names: Vec<String> = cfg.custom_themes.keys().cloned().collect();
        egui::ComboBox::from_label("theme")
            .selected_text(&theme_name)
            .width(180.0)
            .show_ui(ui, |ui| {
                for name in Theme::builtin_names() {
                    ui.selectable_value(&mut cfg.general.theme, name.to_string(), name);
                }
                for name in &custom_names {
                    ui.selectable_value(&mut cfg.general.theme, name.clone(), name);
                }
            });
        let active = Theme::resolve(&cfg.general.theme, &cfg.custom_themes);
        ui.horizontal_wrapped(|ui| {
            theme_swatch(ui, active.overlay, "overlay");
            theme_swatch(ui, active.accent, "accent");
            theme_swatch(ui, active.grid, "grid");
            theme_swatch(ui, active.label, "label");
            theme_swatch(ui, active.hint_bg, "hint");
            theme_swatch(ui, active.bg, "bg");
        });
        if ui
            .button(if self.theme_editor_open {
                "Close theme editor"
            } else {
                "Create your own theme"
            })
            .clicked()
        {
            self.theme_editor_open = !self.theme_editor_open;
            if self.theme_editor_open {
                let a = Theme::resolve(&cfg.general.theme, &cfg.custom_themes);
                self.theme_draft_name = if Theme::builtin_names().contains(&a.name.as_str()) {
                    "my theme".to_string()
                } else {
                    a.name.clone()
                };
                self.theme_draft = a;
            }
        }
        if self.theme_editor_open {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("name");
                ui.add(
                    egui::TextEdit::singleline(&mut self.theme_draft_name).desired_width(120.0),
                );
            });
            color_row(ui, "overlay tint", &mut self.theme_draft.overlay);
            color_row(ui, "grid line", &mut self.theme_draft.grid);
            color_row(ui, "cell label", &mut self.theme_draft.label);
            color_row(ui, "accent", &mut self.theme_draft.accent);
            color_row(ui, "hint bar", &mut self.theme_draft.hint_bg);
            color_row(ui, "hint text", &mut self.theme_draft.hint_text);
            color_row(ui, "status", &mut self.theme_draft.status);
            color_row(ui, "window bg", &mut self.theme_draft.bg);
            color_row(ui, "panel bg", &mut self.theme_draft.panel);
            color_row(ui, "text", &mut self.theme_draft.text);
            ui.checkbox(&mut self.theme_draft.dark, "dark visuals");
            ui.horizontal(|ui| {
                if ui.button("Save as theme").clicked() {
                    let name = self.theme_draft_name.trim().to_string();
                    if !name.is_empty() {
                        self.theme_draft.name = name.clone();
                        cfg.custom_themes
                            .insert(name.clone(), self.theme_draft.clone());
                        cfg.general.theme = name;
                    }
                }
                if cfg.custom_themes.contains_key(&cfg.general.theme) {
                    if ui.button("Delete this theme").clicked() {
                        cfg.custom_themes.remove(&cfg.general.theme);
                        cfg.general.theme = "dark".to_string();
                    }
                }
            });
            ui.label(
                egui::RichText::new(
                    "Tip: exact colors can also be edited in config.toml under [custom_themes.<name>].",
                )
                .weak()
                .small(),
            );
        }

        // ---- Vim mode ----
        ui.add_space(8.0);
        ui.separator();
        ui.label(egui::RichText::new("Vim mode").strong());
        ui.checkbox(&mut cfg.vim.enabled, "Enable vim-style control after grid zoom");
        if cfg.vim.enabled {
            ui.label("hjkl move · H/J/K/L fast move · m/,. click · e/y scroll · v drag");
        }

        // ---- Grid ----
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

        // ---- Vim movement ----
        ui.add_space(8.0);
        ui.separator();
        ui.label(egui::RichText::new("Vim movement").strong());
        ui.add(egui::Slider::new(&mut cfg.general.move_step, 1..=100).text("hjkl step (px)"));
        ui.add(egui::Slider::new(&mut cfg.general.move_fast_step, 10..=500).text("HJKL step (px)"));

        // ---- Hotkeys ----
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
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                let new_cfg = cfg.clone();
                self.controller.config = new_cfg;
                let result = self.controller.config.save();
                match result {
                    Ok(path) => {
                        self.settings_saved_hint = Some(format!("saved to {}", path.display()))
                    }
                    Err(e) => self.settings_saved_hint = Some(format!("save failed: {e}")),
                }
            }
            if ui.button("Cancel").clicked() {
                self.settings_open = false;
            }
        });
        if let Some(hint) = &self.settings_saved_hint {
            ui.label(hint);
        }
    }
}

fn theme_swatch(ui: &mut egui::Ui, rgba: Rgba, label: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 12.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 2.0, to_color32(rgba));
    ui.painter().rect_stroke(
        rect,
        2.0,
        egui::Stroke::new(1.0_f32, Color32::from_gray(90)),
        egui::StrokeKind::Inside,
    );
    ui.label(label);
}

fn color_row(ui: &mut egui::Ui, label: &str, color: &mut Rgba) {
    ui.horizontal(|ui| {
        ui.label(format!("{label:>12}"));
        let mut c = to_color32(*color);
        if ui.color_edit_button_srgba(&mut c).changed() {
            *color = Rgba {
                r: c.r(),
                g: c.g(),
                b: c.b(),
                a: c.a(),
            };
        }
        let mut hex = color.to_hex();
        if ui
            .add(egui::TextEdit::singleline(&mut hex).desired_width(82.0))
            .changed()
        {
            if let Some(v) = Rgba::from_hex(&hex) {
                *color = v;
            }
        }
    });
}

fn capture_monitor_bg(
    ctx: &egui::Context,
    monitor: &mokey_backend::platform::MonitorInfo,
) -> Option<TextureHandle> {
    let image = mokey_backend::screen::capture_region(monitor.rect).ok()?;
    let mut rgba = Vec::with_capacity(image.bgra.len());
    for px in image.bgra.chunks_exact(4) {
        rgba.extend_from_slice(&[px[2], px[1], px[0], 255]);
    }
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [image.width as usize, image.height as usize],
        &rgba,
    );
    Some(ctx.load_texture("hud-bg", color_image, TextureOptions::LINEAR))
}

impl eframe::App for MokeyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.flush_pending();
        self.apply_visuals(ctx);
        self.drain_hotkeys(ctx);
        self.handle_hud_input(ctx);
        self.drain_global_keys(ctx);
        self.draw_hud(ctx);
        self.settings_window(ctx);
        if !self.hud_visible {
            self.move_hud_offscreen(ctx);
        }
        if self.hud_visible {
            let now = Instant::now();
            if self.focus_log_at.map_or(true, |t| now - t > Duration::from_millis(500)) {
                self.focus_log_at = Some(now);
                let focused = ctx.input(|i| i.viewport().focused);
                dbg(format!("EGUI focus: {focused:?}"));
            }
        } else {
            self.focus_log_at = None;
        }
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}
