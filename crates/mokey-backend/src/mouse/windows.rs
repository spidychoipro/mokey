use crate::error::BackendError;
use crate::mouse::{MouseBackend, MouseButton};

pub struct EnigoMouse {
    enigo: enigo::Enigo,
}

impl EnigoMouse {
    pub fn new() -> Result<EnigoMouse, BackendError> {
        let enigo = enigo::Enigo::new(&enigo::Settings::default())
            .map_err(|e| BackendError::Input(e.to_string()))?;
        Ok(EnigoMouse { enigo })
    }
}

fn to_button(b: MouseButton) -> enigo::Button {
    match b {
        MouseButton::Left => enigo::Button::Left,
        MouseButton::Middle => enigo::Button::Middle,
        MouseButton::Right => enigo::Button::Right,
    }
}

impl MouseBackend for EnigoMouse {
    fn location(&self) -> Result<(i32, i32), BackendError> {
        use enigo::Mouse;
        self.enigo
            .location()
            .map_err(|e| BackendError::Input(e.to_string()))
    }

    fn move_to(&mut self, x: i32, y: i32) -> Result<(), BackendError> {
        use enigo::{Coordinate, Mouse};
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| BackendError::Input(e.to_string()))
    }

    fn move_rel(&mut self, dx: i32, dy: i32) -> Result<(), BackendError> {
        use enigo::{Coordinate, Mouse};
        self.enigo
            .move_mouse(dx, dy, Coordinate::Rel)
            .map_err(|e| BackendError::Input(e.to_string()))
    }

    fn click(&mut self, button: MouseButton) -> Result<(), BackendError> {
        use enigo::{Direction, Mouse};
        self.enigo
            .button(to_button(button), Direction::Click)
            .map_err(|e| BackendError::Input(e.to_string()))
    }

    fn scroll(&mut self, lines: i32) -> Result<(), BackendError> {
        use enigo::{Axis, Mouse};
        self.enigo
            .scroll(lines, Axis::Vertical)
            .map_err(|e| BackendError::Input(e.to_string()))
    }

    fn button(&mut self, button: MouseButton, press: bool) -> Result<(), BackendError> {
        use enigo::{Direction, Mouse};
        let dir = if press {
            Direction::Press
        } else {
            Direction::Release
        };
        self.enigo
            .button(to_button(button), dir)
            .map_err(|e| BackendError::Input(e.to_string()))
    }
}
