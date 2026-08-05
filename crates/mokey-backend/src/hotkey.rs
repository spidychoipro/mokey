use crate::error::BackendError;
use mokey_core::{KeyEvent, MokeyKey};
use rdev::{Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyId {
    Trigger,
    Settings,
}

pub struct GlobalInput {
    pub hotkeys: mpsc::Receiver<HotkeyId>,
    /// Forwarded key presses, only produced while `capture` is set (dragging).
    pub keys: mpsc::Receiver<KeyEvent>,
    /// When true, all key presses are forwarded to the `keys` channel.
    pub capture: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct Hotkey {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    pub key: Key,
}

impl Hotkey {
    /// Parse a spec like "Ctrl+Alt+Space" or "Ctrl+Alt+S".
    pub fn parse(spec: &str) -> Result<Hotkey, BackendError> {
        let parts: Vec<&str> = spec.split('+').map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
        if parts.len() < 2 {
            return Err(BackendError::InvalidHotkey(spec.into()));
        }
        let mut h = Hotkey { ctrl: false, alt: false, shift: false, meta: false, key: Key::Unknown(0) };
        let mut main: Option<Key> = None;
        for part in &parts {
            let lower = part.to_ascii_lowercase();
            match lower.as_str() {
                "ctrl" | "control" => h.ctrl = true,
                "alt" | "option" => h.alt = true,
                "shift" => h.shift = true,
                "meta" | "super" | "win" | "cmd" => h.meta = true,
                _ => {
                    if main.is_some() {
                        return Err(BackendError::InvalidHotkey(spec.into()));
                    }
                    main = Some(parse_key(part).ok_or_else(|| BackendError::InvalidHotkey(spec.into()))?);
                }
            }
        }
        h.key = main.ok_or_else(|| BackendError::InvalidHotkey(spec.into()))?;
        Ok(h)
    }

    pub fn matches(&self, ctrl: bool, alt: bool, shift: bool, meta: bool) -> bool {
        self.ctrl == ctrl && self.alt == alt && self.shift == shift && self.meta == meta
    }
}

fn parse_key(s: &str) -> Option<Key> {
    let lower = s.to_ascii_lowercase();
    match lower.as_str() {
        "space" => return Some(Key::Space),
        "enter" | "return" => return Some(Key::Return),
        "tab" => return Some(Key::Tab),
        "esc" | "escape" => return Some(Key::Escape),
        _ => {}
    }
    let mut chars = lower.chars();
    if let Some(c) = chars.next() {
        if chars.next().is_none() {
            if c.is_ascii_digit() {
                let d = c.to_digit(10)?;
                return Some(match d {
                    0 => Key::Num0,
                    1 => Key::Num1,
                    2 => Key::Num2,
                    3 => Key::Num3,
                    4 => Key::Num4,
                    5 => Key::Num5,
                    6 => Key::Num6,
                    7 => Key::Num7,
                    8 => Key::Num8,
                    9 => Key::Num9,
                    _ => return None,
                });
            }
            if c.is_ascii_alphabetic() {
                let idx = c as u8 - b'a';
                let key = match idx {
                    0 => Key::KeyA,
                    1 => Key::KeyB,
                    2 => Key::KeyC,
                    3 => Key::KeyD,
                    4 => Key::KeyE,
                    5 => Key::KeyF,
                    6 => Key::KeyG,
                    7 => Key::KeyH,
                    8 => Key::KeyI,
                    9 => Key::KeyJ,
                    10 => Key::KeyK,
                    11 => Key::KeyL,
                    12 => Key::KeyM,
                    13 => Key::KeyN,
                    14 => Key::KeyO,
                    15 => Key::KeyP,
                    16 => Key::KeyQ,
                    17 => Key::KeyR,
                    18 => Key::KeyS,
                    19 => Key::KeyT,
                    20 => Key::KeyU,
                    21 => Key::KeyV,
                    22 => Key::KeyW,
                    23 => Key::KeyX,
                    24 => Key::KeyY,
                    25 => Key::KeyZ,
                    _ => return None,
                };
                return Some(key);
            }
        }
    }
    None
}

/// Spawn a global key listener thread. Returns a `GlobalInput` with a hotkey
/// receiver and a key-event receiver used while dragging.
pub fn spawn(
    trigger: &str,
    settings: &str,
) -> Result<GlobalInput, BackendError> {
    let trigger = Hotkey::parse(trigger)?;
    let settings = Hotkey::parse(settings)?;
    let (hk_tx, hk_rx) = mpsc::channel();
    let (key_tx, key_rx) = mpsc::channel();
    let capture = Arc::new(AtomicBool::new(false));
    let capture_for_thread = capture.clone();

    std::thread::spawn(move || {
        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut meta = false;
        let capture = capture_for_thread;

        let is_ctrl = |k: Key| matches!(k, Key::ControlLeft | Key::ControlRight);
        let is_alt = |k: Key| matches!(k, Key::Alt | Key::AltGr);
        let is_shift = |k: Key| matches!(k, Key::ShiftLeft | Key::ShiftRight);
        let is_meta = |k: Key| matches!(k, Key::MetaLeft | Key::MetaRight);

        let on_key = move |event: Event,
                           ctrl: &mut bool,
                           alt: &mut bool,
                           shift: &mut bool,
                           meta: &mut bool| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    if is_ctrl(key) {
                        *ctrl = true;
                        return;
                    }
                    if is_alt(key) {
                        *alt = true;
                        return;
                    }
                    if is_shift(key) {
                        *shift = true;
                        return;
                    }
                    if is_meta(key) {
                        *meta = true;
                        return;
                    }
                    let mut matched_hotkey = false;
                    for (spec, id) in [(&trigger, HotkeyId::Trigger), (&settings, HotkeyId::Settings)] {
                        if key == spec.key && spec.matches(*ctrl, *alt, *shift, *meta) {
                            let _ = hk_tx.send(id);
                            matched_hotkey = true;
                        }
                    }
                    if !matched_hotkey && capture.load(Ordering::Relaxed) {
                        if let Some(ke) = map_key(key, *shift) {
                            let _ = key_tx.send(ke);
                        }
                    }
                }
                EventType::KeyRelease(key) => {
                    if is_ctrl(key) {
                        *ctrl = false;
                    }
                    if is_alt(key) {
                        *alt = false;
                    }
                    if is_shift(key) {
                        *shift = false;
                    }
                    if is_meta(key) {
                        *meta = false;
                    }
                }
                _ => {}
            }
        };

        let result = rdev::listen(move |event| {
            on_key(event, &mut ctrl, &mut alt, &mut shift, &mut meta)
        });
        if let Err(e) = result {
            eprintln!("mokey: global key listener failed: {e:?}");
        }
    });

    Ok(GlobalInput {
        hotkeys: hk_rx,
        keys: key_rx,
        capture,
    })
}

fn map_key(key: Key, shift: bool) -> Option<KeyEvent> {
    use Key as K;
    let mk = match key {
        K::Num0 => MokeyKey::Digit(0),
        K::Num1 => MokeyKey::Digit(1),
        K::Num2 => MokeyKey::Digit(2),
        K::Num3 => MokeyKey::Digit(3),
        K::Num4 => MokeyKey::Digit(4),
        K::Num5 => MokeyKey::Digit(5),
        K::Num6 => MokeyKey::Digit(6),
        K::Num7 => MokeyKey::Digit(7),
        K::Num8 => MokeyKey::Digit(8),
        K::Num9 => MokeyKey::Digit(9),
        K::KeyH => MokeyKey::H,
        K::KeyJ => MokeyKey::J,
        K::KeyK => MokeyKey::K,
        K::KeyL => MokeyKey::L,
        K::KeyM => MokeyKey::M,
        K::KeyE => MokeyKey::E,
        K::KeyY => MokeyKey::Y,
        K::KeyV => MokeyKey::V,
        K::Space => MokeyKey::Space,
        K::Return => MokeyKey::Enter,
        K::Backspace => MokeyKey::Backspace,
        K::Escape => MokeyKey::Escape,
        K::Comma => MokeyKey::Comma,
        K::Dot => MokeyKey::Period,
        _ => MokeyKey::Other,
    };
    if mk == MokeyKey::Other {
        return None;
    }
    Some(KeyEvent { key: mk, shift })
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn injected_trigger_hotkey_is_detected() {
        let input = spawn("Ctrl+Alt+Space", "Ctrl+Alt+S").expect("spawn listener");
        std::thread::sleep(Duration::from_millis(400));
        let mut enigo = enigo::Enigo::new(&enigo::Settings::default()).expect("enigo init");
        use enigo::Keyboard;
        use enigo::Direction::{Press, Release};
        use enigo::Key as EKey;

        enigo.key(EKey::Control, Press).ok();
        enigo.key(EKey::Alt, Press).ok();
        enigo.key(EKey::Space, Press).ok();
        enigo.key(EKey::Space, Release).ok();
        enigo.key(EKey::Alt, Release).ok();
        enigo.key(EKey::Control, Release).ok();

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            match input.hotkeys.try_recv() {
                Ok(HotkeyId::Trigger) => return,
                Ok(_) => panic!("wrong hotkey"),
                Err(mpsc::TryRecvError::Empty) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(e) => panic!("hotkey not received: {e:?}"),
            }
        }
    }
}
