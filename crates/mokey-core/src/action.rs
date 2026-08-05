#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Zoom(u32),
    ZoomOut,
    Cancel,
    ClickLeft,
    ClickRight,
    ClickMiddle,
    MoveCursor(i32, i32),
    Scroll(i32),
    ToggleDrag,
    Noop,
}
