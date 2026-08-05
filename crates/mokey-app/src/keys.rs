use egui::{Event, Key as EguiKey};
use mokey_core::MokeyKey;

/// Map an egui key to a mokey key. Unused keys map to `Other`.
pub fn map_key(key: EguiKey) -> MokeyKey {
    use EguiKey as K;
    match key {
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
        K::Enter => MokeyKey::Enter,
        K::Space => MokeyKey::Space,
        K::Backspace => MokeyKey::Backspace,
        K::Escape => MokeyKey::Escape,
        K::Comma => MokeyKey::Comma,
        K::Period => MokeyKey::Period,
        K::H => MokeyKey::H,
        K::J => MokeyKey::J,
        K::K => MokeyKey::K,
        K::L => MokeyKey::L,
        K::M => MokeyKey::M,
        K::E => MokeyKey::E,
        K::Y => MokeyKey::Y,
        K::V => MokeyKey::V,
        _ => MokeyKey::Other,
    }
}

/// True if the given egui event is a key press we care about.
pub fn is_key_press(event: &Event) -> bool {
    matches!(event, Event::Key { pressed: true, .. })
}
