use crate::config::Config;
use crate::lastfm::LastFm;
use crate::mpris::TrackInfo;
use iced::Color;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use tray_icon::TrayIcon;
use std::collections::HashMap;

pub struct App {
    pub config: Config,
    pub config_path: Option<std::path::PathBuf>,
    pub parsed_colors: ParsedColors,
    pub current_track: Option<TrackInfo>,
    pub lastfm: Option<Arc<LastFm>>,
    pub auth_token: Option<last_fm_rs::AuthToken>,
    pub auth_url: Option<String>,
    pub error_message: Option<String>,
    pub now_playing_sent: bool,
    pub scrobble_sent: bool,
    pub track_start_time: Option<DateTime<Utc>>,
    pub last_notified_track: Option<String>,
    pub auth_attempts: u32,
    pub last_auth_attempt: Option<DateTime<Utc>>,
    pub track_image_bytes: Option<Vec<u8>>,
    pub image_cache: HashMap<String, Vec<u8>>,
    pub all_players: Vec<String>,
    pub track_verified: bool,
    pub track_total_played_secs: u64,
    pub last_resume_time: Option<DateTime<Utc>>,
    pub visible: bool,
    pub _tray_icon: Arc<TrayIcon>,
}

#[derive(Debug, Clone, Copy)]
pub struct ParsedColors {
    pub base: Color,
    pub slightly_lighter: Color,
    pub accent_grey: Color,
    pub bright: Color,
    pub text: Color,
}

impl ParsedColors {
    pub fn from_config(config: &Config) -> Self {
        let colors = &config.ui.color_scheme;
        Self {
            base: colors.base.parse().unwrap_or(Color::BLACK),
            slightly_lighter: colors.slightly_lighter.parse().unwrap_or(Color::from_rgb(0.2, 0.2, 0.3)),
            accent_grey: colors.accent_grey.parse().unwrap_or(Color::from_rgb(0.4, 0.4, 0.5)),
            bright: colors.bright.parse().unwrap_or(Color::WHITE),
            text: colors.text.parse().unwrap_or(Color::WHITE),
        }
    }
}

impl App {
    pub fn new(config_path: Option<std::path::PathBuf>, tray_icon: Arc<TrayIcon>) -> Self {
        let config = Config::load(config_path.clone()).unwrap_or_default();
        let parsed_colors = ParsedColors::from_config(&config);
        let lastfm = if let (Some(session_key), api_key, api_secret) = (
            &config.lastfm.session_key,
            &config.lastfm.api_key,
            &config.lastfm.api_secret,
        ) {
            if !api_key.is_empty() && !api_secret.is_empty() {
                let mut lfm = LastFm::new(api_key.clone(), api_secret.clone());
                lfm = lfm.with_session_key(session_key.clone());
                Some(Arc::new(lfm))
            } else {
                None
            }
        } else {
            None
        };

        Self {
            config,
            config_path,
            parsed_colors,
            current_track: None,
            lastfm,
            auth_token: None,
            auth_url: None,
            error_message: None,
            now_playing_sent: false,
            scrobble_sent: false,
            track_start_time: None,
            last_notified_track: None,
            auth_attempts: 0,
            last_auth_attempt: None,
            track_image_bytes: None,
            image_cache: HashMap::new(),
            all_players: Vec::new(),
            track_verified: false,
            track_total_played_secs: 0,
            last_resume_time: None,
            visible: true,
            _tray_icon: tray_icon,
        }
    }
}
