use crate::error::BackendError;
use crate::mouse::{MouseBackend, MouseButton};

/// Placeholder for the Linux backends (Hyprland / KDE / GNOME).
/// Implemented in Phase 2 and Phase 3.
pub struct UnsupportedMouse;

impl MouseBackend for UnsupportedMouse {
    fn location(&self) -> Result<(i32, i32), BackendError> {
        Err(BackendError::UnsupportedPlatform(
            "mokey has not been ported to this Linux desktop yet".into(),
        ))
    }

    fn move_to(&mut self, _x: i32, _y: i32) -> Result<(), BackendError> {
        Err(BackendError::UnsupportedPlatform(
            "mokey has not been ported to this Linux desktop yet".into(),
        ))
    }

    fn move_rel(&mut self, _dx: i32, _dy: i32) -> Result<(), BackendError> {
        Err(BackendError::UnsupportedPlatform(
            "mokey has not been ported to this Linux desktop yet".into(),
        ))
    }

    fn click(&mut self, _button: MouseButton) -> Result<(), BackendError> {
        Err(BackendError::UnsupportedPlatform(
            "mokey has not been ported to this Linux desktop yet".into(),
        ))
    }

    fn scroll(&mut self, _lines: i32) -> Result<(), BackendError> {
        Err(BackendError::UnsupportedPlatform(
            "mokey has not been ported to this Linux desktop yet".into(),
        ))
    }

    fn button(&mut self, _button: MouseButton, _press: bool) -> Result<(), BackendError> {
        Err(BackendError::UnsupportedPlatform(
            "mokey has not been ported to this Linux desktop yet".into(),
        ))
    }
}
