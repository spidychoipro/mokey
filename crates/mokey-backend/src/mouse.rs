use crate::error::BackendError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// Platform mouse control. All coordinates are in physical pixels.
pub trait MouseBackend: Send {
    /// Current cursor position in physical pixels.
    fn location(&self) -> Result<(i32, i32), BackendError>;
    /// Move the cursor to an absolute position.
    fn move_to(&mut self, x: i32, y: i32) -> Result<(), BackendError>;
    /// Move the cursor by a relative offset.
    fn move_rel(&mut self, dx: i32, dy: i32) -> Result<(), BackendError>;
    /// Press and release a button (a click).
    fn click(&mut self, button: MouseButton) -> Result<(), BackendError>;
    /// Scroll vertically. Positive = down, negative = up, in lines.
    fn scroll(&mut self, lines: i32) -> Result<(), BackendError>;
    /// Press (true) or release (false) a button, used for dragging.
    fn button(&mut self, button: MouseButton, press: bool) -> Result<(), BackendError>;
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::EnigoMouse;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::UnsupportedMouse;

#[cfg(not(any(windows, unix)))]
compile_error!("mokey only supports Windows and Linux");

/// Create the mouse backend for the current platform.
#[cfg(windows)]
pub fn create() -> Result<Box<dyn MouseBackend>, BackendError> {
    Ok(Box::new(EnigoMouse::new()?))
}

/// Create the mouse backend for the current platform.
#[cfg(unix)]
pub fn create() -> Result<Box<dyn MouseBackend>, BackendError> {
    Ok(Box::new(UnsupportedMouse))
}
