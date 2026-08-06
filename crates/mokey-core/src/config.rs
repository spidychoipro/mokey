use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: General,
    pub vim: Vim,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct General {
    /// Hotkey that summons the grid overlay. Format: "Ctrl+Alt+Space"
    pub trigger_hotkey: String,
    /// Hotkey that opens the settings window. Format: "Ctrl+Alt+S"
    pub settings_hotkey: String,
    /// Number of divisions per axis for the grid (3 -> 3x3 grid with labels 1-9).
    pub grid_size: u32,
    /// Maximum zoom depth. Deeper levels are smaller, more precise regions.
    pub max_depth: u32,
    /// Auto click when the maximum zoom depth is reached.
    pub auto_click: bool,
    /// Background opacity of the grid overlay (0.0 ..= 1.0).
    pub overlay_bg_opacity: f32,
    /// Pixel step for lowercase hjkl cursor movement (vim mode).
    pub move_step: i32,
    /// Pixel step for uppercase HJKL cursor movement (vim mode).
    pub move_fast_step: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Vim {
    /// Default OFF. Users opt in via settings.
    pub enabled: bool,
}

impl Default for Vim {
    fn default() -> Self {
        Vim { enabled: false }
    }
}

impl Default for General {
    fn default() -> Self {
        General {
            trigger_hotkey: "Ctrl+Alt+Space".to_string(),
            settings_hotkey: "Ctrl+Alt+S".to_string(),
            grid_size: 3,
            max_depth: 6,
            auto_click: false,
            overlay_bg_opacity: 0.45,
            move_step: 10,
            move_fast_step: 100,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: General {
                trigger_hotkey: "Ctrl+Alt+Space".to_string(),
                settings_hotkey: "Ctrl+Alt+S".to_string(),
                grid_size: 3,
                max_depth: 6,
                auto_click: false,
                overlay_bg_opacity: 0.45,
                move_step: 10,
                move_fast_step: 100,
            },
            vim: Vim { enabled: false },
        }
    }
}

impl General {
    pub fn overlay_opacity(&self) -> f32 {
        self.overlay_bg_opacity.clamp(0.0, 1.0)
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf, ConfigError> {
        dirs::config_dir()
            .map(|d| d.join("mokey"))
            .ok_or(ConfigError::NoConfigDir)
    }

    pub fn config_path() -> Result<PathBuf, ConfigError> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Config, ConfigError> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let raw = std::fs::read_to_string(&path)?;
        let cfg: Config = toml::from_str(&raw)?;
        Ok(cfg)
    }

    pub fn save(&self) -> Result<PathBuf, ConfigError> {
        let dir = Self::config_dir()?;
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("config.toml");
        let raw = toml::to_string_pretty(self)?;
        std::fs::write(&path, raw)?;
        Ok(path)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("could not determine the config directory")]
    NoConfigDir,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("toml serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
}
