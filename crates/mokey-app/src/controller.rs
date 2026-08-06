use mokey_backend::mouse::{MouseBackend, MouseButton};
use mokey_core::input;
use mokey_core::{Action, Config, GridSession, KeyEvent, Rect};

#[derive(Debug, Default)]
pub struct ExecOutcome {
    /// Session is over and the HUD should close.
    pub finished: bool,
    /// The HUD window must be hidden before any pointer action runs.
    pub hide_hud: bool,
    /// The HUD window should be shown again (e.g. after drag ends).
    pub show_hud: bool,
    /// Global key capture should be enabled (used while dragging).
    pub capture_keys: bool,
    /// A click to perform after the HUD is hidden.
    pub click: Option<MouseButton>,
    /// Press (true) or release (false) the left button after hiding.
    pub press_drag: Option<bool>,
}

pub struct Controller {
    pub mouse: Box<dyn MouseBackend>,
    pub config: Config,
    pub session: Option<GridSession>,
    pub dragging: bool,
}

impl Controller {
    pub fn new(config: Config, mouse: Box<dyn MouseBackend>) -> Controller {
        Controller {
            mouse,
            config,
            session: None,
            dragging: false,
        }
    }

    pub fn start_session(&mut self, monitor: Rect) {
        if self.session.is_some() {
            return;
        }
        self.dragging = false;
        let session = GridSession::start(monitor, self.config.general.grid_size, self.config.general.max_depth);
        self.session = Some(session);
        self.move_to_region_center();
    }

    pub fn move_to_region_center(&mut self) {
        if let Some(s) = &self.session {
            let p = s.click_point();
            let _ = self.mouse.move_to(p.x, p.y);
        }
    }

    /// Handle a key event, returning the outcome the app should act on.
    pub fn process(&mut self, ev: KeyEvent) -> ExecOutcome {
        let Some(session) = self.session.as_mut() else {
            return ExecOutcome::default();
        };
        let action = input::handle_key(session, &self.config, ev);
        self.execute(action)
    }

    pub fn execute(&mut self, action: Action) -> ExecOutcome {
        let mut out = ExecOutcome::default();
        match action {
            Action::Zoom(_) => {
                self.move_to_region_center();
                let auto = self.config.general.auto_click;
                let at_max = self.session.as_ref().map_or(false, |s| !s.can_zoom());
                if auto && at_max {
                    out.hide_hud = true;
                    out.click = Some(MouseButton::Left);
                    out.finished = true;
                }
            }
            Action::ZoomOut => self.move_to_region_center(),
            Action::ClickLeft => {
                self.move_to_region_center();
                out.hide_hud = true;
                out.click = Some(MouseButton::Left);
                out.finished = true;
            }
            Action::ClickMiddle => {
                self.move_to_region_center();
                out.hide_hud = true;
                out.click = Some(MouseButton::Middle);
                out.finished = true;
            }
            Action::ClickRight => {
                self.move_to_region_center();
                out.hide_hud = true;
                out.click = Some(MouseButton::Right);
                out.finished = true;
            }
            Action::MoveCursor(dx, dy) => {
                let _ = self.mouse.move_rel(dx, dy);
            }
            Action::Scroll(n) => {
                let _ = self.mouse.scroll(n);
            }
            Action::ToggleDrag => {
                if self.dragging {
                    out.press_drag = Some(false);
                    out.show_hud = true;
                    self.dragging = false;
                } else {
                    out.hide_hud = true;
                    out.press_drag = Some(true);
                    out.capture_keys = true;
                    self.dragging = true;
                }
            }
            Action::Cancel => {
                if self.dragging {
                    out.press_drag = Some(false);
                    self.dragging = false;
                }
                out.hide_hud = true;
                out.finished = true;
            }
            Action::Noop => {}
        }

        if out.finished {
            self.session = None;
            self.dragging = false;
        }
        out
    }

    /// Perform pointer actions that must run while the HUD is hidden.
    pub fn apply_hidden_actions(&mut self, out: &ExecOutcome) {
        if let Some(button) = out.click {
            let _ = self.mouse.click(button);
        }
        if let Some(press) = out.press_drag {
            let _ = self.mouse.button(MouseButton::Left, press);
        }
    }
}
