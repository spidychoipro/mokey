#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("input backend error: {0}")]
    Input(String),
    #[error("hotkey error: {0}")]
    Hotkey(String),
    #[error("invalid hotkey spec: {0}")]
    InvalidHotkey(String),
}
