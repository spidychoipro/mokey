use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// RGBA color. Serialized as hex strings like `#aabbcc` or `#aabbccdd`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Rgba {
        Rgba { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Rgba {
        Rgba { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Option<Rgba> {
        let h = hex.trim().trim_start_matches('#');
        let val = u32::from_str_radix(h, 16).ok()?;
        match h.len() {
            6 => Some(Rgba {
                r: ((val >> 16) & 0xff) as u8,
                g: ((val >> 8) & 0xff) as u8,
                b: (val & 0xff) as u8,
                a: 255,
            }),
            8 => Some(Rgba {
                r: ((val >> 24) & 0xff) as u8,
                g: ((val >> 16) & 0xff) as u8,
                b: ((val >> 8) & 0xff) as u8,
                a: (val & 0xff) as u8,
            }),
            _ => None,
        }
    }

    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Rgba::rgba(0, 0, 0, 255)
    }
}

impl Serialize for Rgba {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Rgba {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Rgba::from_hex(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid color: {s}")))
    }
}

/// A HUD/UI color palette. `overlay` is tinted over the desktop screenshot;
/// its alpha is driven by `general.overlay_bg_opacity`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    #[serde(default)]
    pub name: String,
    /// Dark tint laid over the desktop screenshot.
    #[serde(default)]
    pub overlay: Rgba,
    /// Cell border color in the grid.
    #[serde(default)]
    pub grid: Rgba,
    /// Cell number label color.
    #[serde(default)]
    pub label: Rgba,
    /// Current region outline (the zoom area).
    #[serde(default)]
    pub accent: Rgba,
    /// Bottom hint bar background.
    #[serde(default)]
    pub hint_bg: Rgba,
    /// Bottom hint bar text.
    #[serde(default)]
    pub hint_text: Rgba,
    /// Top-left status text.
    #[serde(default)]
    pub status: Rgba,
    /// Settings window background.
    #[serde(default)]
    pub bg: Rgba,
    /// Settings window panel/card background.
    #[serde(default)]
    pub panel: Rgba,
    /// Settings window text.
    #[serde(default)]
    pub text: Rgba,
    /// Use egui dark visuals (false = light).
    #[serde(default = "default_dark")]
    pub dark: bool,
}

fn default_dark() -> bool {
    true
}

impl Theme {
    pub fn builtin() -> Vec<Theme> {
        vec![Theme::dark(), Theme::dracula(), Theme::nord(), Theme::light()]
    }

    pub fn builtin_names() -> Vec<&'static str> {
        vec!["dark", "dracula", "nord", "light"]
    }

    pub fn dark() -> Theme {
        Theme {
            name: "dark".into(),
            overlay: Rgba::rgb(0, 0, 0),
            grid: Rgba::rgba(255, 255, 255, 120),
            label: Rgba::rgba(255, 255, 255, 220),
            accent: Rgba::rgb(80, 220, 255),
            hint_bg: Rgba::rgba(0, 0, 0, 140),
            hint_text: Rgba::rgb(255, 255, 255),
            status: Rgba::rgb(255, 255, 255),
            bg: Rgba::rgb(24, 24, 27),
            panel: Rgba::rgb(39, 39, 42),
            text: Rgba::rgb(228, 228, 231),
            dark: true,
        }
    }

    pub fn dracula() -> Theme {
        Theme {
            name: "dracula".into(),
            overlay: Rgba::rgb(40, 42, 54),
            grid: Rgba::rgb(68, 71, 90),
            label: Rgba::rgb(248, 248, 242),
            accent: Rgba::rgb(189, 147, 249),
            hint_bg: Rgba::rgb(33, 34, 44),
            hint_text: Rgba::rgb(248, 248, 242),
            status: Rgba::rgb(98, 114, 164),
            bg: Rgba::rgb(40, 42, 54),
            panel: Rgba::rgb(33, 34, 44),
            text: Rgba::rgb(248, 248, 242),
            dark: true,
        }
    }

    pub fn nord() -> Theme {
        Theme {
            name: "nord".into(),
            overlay: Rgba::rgb(46, 52, 64),
            grid: Rgba::rgb(76, 86, 106),
            label: Rgba::rgb(216, 222, 233),
            accent: Rgba::rgb(136, 192, 208),
            hint_bg: Rgba::rgb(59, 66, 82),
            hint_text: Rgba::rgb(236, 239, 244),
            status: Rgba::rgb(129, 161, 193),
            bg: Rgba::rgb(46, 52, 64),
            panel: Rgba::rgb(59, 66, 82),
            text: Rgba::rgb(236, 239, 244),
            dark: true,
        }
    }

    pub fn light() -> Theme {
        Theme {
            name: "light".into(),
            overlay: Rgba::rgb(248, 248, 248),
            grid: Rgba::rgba(0, 0, 0, 64),
            label: Rgba::rgb(0, 0, 0),
            accent: Rgba::rgb(30, 102, 245),
            hint_bg: Rgba::rgba(255, 255, 255, 217),
            hint_text: Rgba::rgb(28, 28, 28),
            status: Rgba::rgb(76, 79, 105),
            bg: Rgba::rgb(239, 241, 245),
            panel: Rgba::rgb(230, 233, 239),
            text: Rgba::rgb(28, 28, 28),
            dark: false,
        }
    }

    /// Resolve the active theme: a custom theme with that name wins,
    /// otherwise a builtin theme, otherwise the default `dark` theme.
    pub fn resolve(name: &str, custom: &BTreeMap<String, Theme>) -> Theme {
        if let Some(t) = custom.get(name) {
            let mut t = t.clone();
            if t.name.is_empty() {
                t.name = name.to_string();
            }
            return t;
        }
        Theme::by_name(name).unwrap_or_else(Theme::dark)
    }

    fn by_name(name: &str) -> Option<Theme> {
        Theme::builtin().into_iter().find(|t| t.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let c = Rgba::rgba(189, 147, 249, 200);
        assert_eq!(Rgba::from_hex(&c.to_hex()), Some(c));
        assert_eq!(Rgba::from_hex("#ff0000"), Some(Rgba::rgb(255, 0, 0)));
        assert_eq!(Rgba::from_hex("bad"), None);
    }

    #[test]
    fn resolve_prefers_custom() {
        let mut custom = BTreeMap::new();
        custom.insert("mine".into(), Theme { name: "".into(), accent: Rgba::rgb(1, 2, 3), ..Theme::dracula() });
        let t = Theme::resolve("mine", &custom);
        assert_eq!(t.name, "mine");
        assert_eq!(t.accent, Rgba::rgb(1, 2, 3));
        assert_eq!(Theme::resolve("nope", &custom).name, "dark");
    }

    #[test]
    fn builtin_dark_is_default() {
        assert_eq!(Theme::resolve("dracula", &BTreeMap::new()).name, "dracula");
        assert_eq!(Theme::resolve("", &BTreeMap::new()).name, "dark");
    }
}
