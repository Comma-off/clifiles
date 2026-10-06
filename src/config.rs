use crate::fs::SortMode;
use ratatui::style::Color;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub show_hidden: bool,
    pub sort: SortModeConfig,
    pub start_dir: StartDir,
    pub colors: Colors,
    pub image: ImageConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_hidden: false,
            sort: SortModeConfig::Name,
            start_dir: StartDir::Home,
            colors: Colors::default(),
            image: ImageConfig::default(),
        }
    }
}

/// Lets a user correct image preview quality/protocol when auto-detection
/// can't get it right — e.g. over SSH, where the local terminal's
/// identifying env vars (`TERM_PROGRAM`, `KITTY_WINDOW_ID`, ...) usually
/// aren't forwarded to the remote shell, so protocol detection has nothing
/// to go on. `cell_width`/`cell_height` fix low-resolution/blurry images:
/// without a terminal query (which we deliberately avoid — see
/// `picker_setup.rs`), the real font cell size in pixels often can't be
/// determined and falls back to a generic guess, capping how much detail an
/// image can show. If your terminal reports a pixel size, set it here.
#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(default)]
pub struct ImageConfig {
    pub protocol: Option<ImageProtocol>,
    pub cell_width: Option<u16>,
    pub cell_height: Option<u16>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageProtocol {
    Kitty,
    Iterm2,
    Sixel,
    Halfblocks,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StartDir {
    #[default]
    Home,
    Current,
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
