use crate::fs::SortMode;
use ratatui::style::Color;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub show_hidden: bool,
    pub sort: SortModeConfig,
    pub colors: Colors,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_hidden: false,
            sort: SortModeConfig::Name,
            colors: Colors::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortModeConfig {
    #[default]
    Name,
    Size,
    Modified,
}

impl From<SortModeConfig> for SortMode {
    fn from(value: SortModeConfig) -> Self {
        match value {
            SortModeConfig::Name => SortMode::Name,
            SortModeConfig::Size => SortMode::Size,
            SortModeConfig::Modified => SortMode::Modified,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Colors {
    pub dir: ColorSpec,
    pub symlink: ColorSpec,
    pub file: ColorSpec,
    pub highlight_fg: Option<ColorSpec>,
    pub highlight_bg: Option<ColorSpec>,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            dir: ColorSpec(Color::Blue),
            symlink: ColorSpec(Color::Cyan),
            file: ColorSpec(Color::White),
            highlight_fg: None,
            highlight_bg: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorSpec(pub Color);

impl<'de> Deserialize<'de> for ColorSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_color(&s)
            .map(ColorSpec)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid color: {s}")))
    }
}

fn parse_color(s: &str) -> Option<Color> {
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        }
        return None;
    }
    Some(match s.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        _ => return None,
    })
}

pub fn config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "clifiles")
        .map(|dirs| dirs.config_dir().join("config.toml"))
}

/// Loads the config from the OS config dir, falling back to defaults if the
/// file is missing or fails to parse (a malformed config should never block
/// startup of a file manager).
pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Config::default();
    };
    toml::from_str(&text).unwrap_or_default()
}
