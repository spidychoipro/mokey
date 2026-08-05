use crate::action::Action;
use crate::config::Config;
use crate::keys::{KeyEvent, MokeyKey};
use crate::state::GridSession;

/// Translate a key event into an action given the current session and config.
///
/// Grid semantics (both modes):
/// - digits 1-9 zoom into the labelled cell while the region can still zoom
/// - Enter / Space = left click at the region center
/// - Backspace = zoom out, Escape = cancel
///
/// Vim mode (opt-in, default OFF) additionally enables:
/// - h/j/k/l cursor movement (+H/J/K/L for larger steps)
/// - e / y scroll down / up
/// - m = left click, `,` = middle click, `.` = right click
/// - v = toggle drag
///
/// At max zoom depth digits become a repeat-count prefix for motion keys
/// (e.g. `3j` moves 3 steps), so both grid zooming and vim counts coexist.
pub fn handle_key(session: &mut GridSession, config: &Config, ev: KeyEvent) -> Action {
    let vim = config.vim.enabled;

    match ev.key {
        MokeyKey::Digit(d) if d == 0 || d > session.grid().label_count() => {
            session.clear_count();
            Action::Noop
        }
        MokeyKey::Digit(d) if vim && !session.can_zoom() => {
            session.push_digit(d);
            Action::Noop
        }
        MokeyKey::Digit(d) => match session.zoom_to(d) {
            Some(_) => Action::Zoom(d),
            None => Action::Noop,
        },

        MokeyKey::Enter | MokeyKey::Space => Action::ClickLeft,
        MokeyKey::Backspace => {
            session.clear_count();
            session.zoom_out();
            Action::ZoomOut
        }
        MokeyKey::Escape => {
            session.clear_count();
            Action::Cancel
        }

        MokeyKey::H if vim => motion(session, config, ev.shift, -1, 0),
        MokeyKey::J if vim => motion(session, config, ev.shift, 0, 1),
        MokeyKey::K if vim => motion(session, config, ev.shift, 0, -1),
        MokeyKey::L if vim => motion(session, config, ev.shift, 1, 0),

        MokeyKey::E if vim => scroll(session, config, 1),
        MokeyKey::Y if vim => scroll(session, config, -1),

        MokeyKey::M if vim => Action::ClickLeft,
        MokeyKey::Comma if vim => Action::ClickMiddle,
        MokeyKey::Period if vim => Action::ClickRight,
        MokeyKey::V if vim => Action::ToggleDrag,

        _ => Action::Noop,
    }
}

fn motion(session: &mut GridSession, config: &Config, shift: bool, dx: i32, dy: i32) -> Action {
    let count = session.take_count() as i32;
    let step = if shift {
        config.general.move_fast_step
    } else {
        config.general.move_step
    };
    Action::MoveCursor(dx * step * count, dy * step * count)
}

fn scroll(session: &mut GridSession, config: &Config, dir: i32) -> Action {
    let _ = config;
    let count = session.take_count() as i32;
    Action::Scroll(dir * count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::keys::MokeyKey;

    fn session() -> GridSession {
        GridSession::start(
            Rect { x: 0, y: 0, w: 1920, h: 1080 },
            3,
            4,
        )
    }

    fn cfg(vim: bool) -> Config {
        let mut c = Config::default();
        c.vim.enabled = vim;
        c
    }

    fn ev(key: MokeyKey) -> KeyEvent {
        KeyEvent { key, shift: false }
    }

    #[test]
    fn digits_zoom_when_grid_active() {
        let mut s = session();
        assert_eq!(handle_key(&mut s, &cfg(false), ev(MokeyKey::Digit(5))), Action::Zoom(5));
        assert_eq!(s.region, Rect { x: 640, y: 360, w: 640, h: 360 });
    }

    #[test]
    fn enter_clicks_center() {
        let mut s = session();
        let _ = s.zoom_to(5);
        assert_eq!(handle_key(&mut s, &cfg(false), ev(MokeyKey::Enter)), Action::ClickLeft);
    }

    #[test]
    fn escape_cancels() {
        let mut s = session();
        assert_eq!(handle_key(&mut s, &cfg(false), ev(MokeyKey::Escape)), Action::Cancel);
    }

    #[test]
    fn backspace_zooms_out() {
        let mut s = session();
        let _ = s.zoom_to(5);
        assert_eq!(handle_key(&mut s, &cfg(false), ev(MokeyKey::Backspace)), Action::ZoomOut);
        assert_eq!(s.region, Rect { x: 0, y: 0, w: 1920, h: 1080 });
    }

    #[test]
    fn vim_keys_are_noop_when_disabled() {
        let mut s = session();
        assert_eq!(handle_key(&mut s, &cfg(false), ev(MokeyKey::J)), Action::Noop);
        assert_eq!(handle_key(&mut s, &cfg(false), ev(MokeyKey::M)), Action::Noop);
    }

    #[test]
    fn vim_motion_moves_cursor() {
        let mut s = session();
        assert_eq!(
            handle_key(&mut s, &cfg(true), ev(MokeyKey::J)),
            Action::MoveCursor(0, 10)
        );
        assert_eq!(
            handle_key(&mut s, &cfg(true), KeyEvent { key: MokeyKey::L, shift: true }),
            Action::MoveCursor(100, 0)
        );
    }

    #[test]
    fn digits_become_counts_at_max_depth() {
        let mut s = session();
        // zoom to max depth
        while s.zoom_to(5).is_some() {}
        assert!(!s.can_zoom());
        assert_eq!(handle_key(&mut s, &cfg(true), ev(MokeyKey::Digit(3))), Action::Noop);
        assert_eq!(
            handle_key(&mut s, &cfg(true), ev(MokeyKey::J)),
            Action::MoveCursor(0, 30)
        );
    }

    #[test]
    fn vim_scroll_uses_count() {
        let mut s = session();
        while s.zoom_to(5).is_some() {}
        let _ = handle_key(&mut s, &cfg(true), ev(MokeyKey::Digit(2)));
        assert_eq!(handle_key(&mut s, &cfg(true), ev(MokeyKey::E)), Action::Scroll(2));
        assert_eq!(handle_key(&mut s, &cfg(true), ev(MokeyKey::Y)), Action::Scroll(-1));
    }

    #[test]
    fn clicks_work_in_vim_mode() {
        let mut s = session();
        assert_eq!(handle_key(&mut s, &cfg(true), ev(MokeyKey::M)), Action::ClickLeft);
        assert_eq!(handle_key(&mut s, &cfg(true), ev(MokeyKey::Comma)), Action::ClickMiddle);
        assert_eq!(handle_key(&mut s, &cfg(true), ev(MokeyKey::Period)), Action::ClickRight);
        assert_eq!(handle_key(&mut s, &cfg(true), ev(MokeyKey::V)), Action::ToggleDrag);
    }
}
