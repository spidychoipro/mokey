#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MokeyKey {
    Digit(u32),
    Enter,
    Space,
    Backspace,
    Escape,
    H,
    J,
    K,
    L,
    M,
    Comma,
    Period,
    E,
    Y,
    V,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: MokeyKey,
    pub shift: bool,
}
