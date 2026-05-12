use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub lastfm: LastFmConfig,
    pub ui: UiConfig,
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastFmConfig {
    pub api_key: String,
    pub api_secret: String,
    pub session_key: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub color_scheme: ColorScheme,
    pub position: Position,
    pub show_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub base: String,
    pub slightly_lighter: String,
    pub accent_grey: String,
    pub bright: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub anchor: Anchor,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub scrobble_enabled: bool,
    pub poll_interval_secs: u64,
    pub ignored_players: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lastfm: LastFmConfig {
                api_key: String::new(),
                api_secret: String::new(),
                session_key: None,
                username: None,
            },
            ui: UiConfig {
                color_scheme: ColorScheme::default(),
                position: Position::default(),
                show_notifications: true,
            },
        general: GeneralConfig {
            scrobble_enabled: true,
            poll_interval_secs: 5,
            ignored_players: vec![],
        },
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            base: "#11111b".to_string(),
            slightly_lighter: "#1e1e2e".to_string(),
            accent_grey: "#6C7086".to_string(),
            bright: "#BAC2DE".to_string(),
            text: "#cdd6f4".to_string(),
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            anchor: Anchor::BottomRight,
            x: 20,
            y: 20,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("traac")
            .join("config.toml")
    }

    pub fn load(custom_path: Option<PathBuf>) -> std::io::Result<Self> {
        let path = custom_path.unwrap_or_else(Self::config_path);
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = std::fs::read_to_string(&path)?;
        match toml::from_str(&content) {
            Ok(config) => Ok(config),
            Err(e) => {
                eprintln!("Warning: Failed to parse config.toml: {}", e);
                eprintln!("         Using default configuration.");
                Ok(Config::default())
            }
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })?;
        std::fs::write(&path, content)
    }
}

