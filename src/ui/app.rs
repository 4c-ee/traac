use crate::config::{Anchor, Config};
use crate::lastfm::LastFm;
use crate::mpris::{find_player_with_ignore, get_current_track, is_playing, TrackInfo};
use iced::widget::{column, container, text, button, row, image};
use iced::{Color, Element, Length, Task};
use iced_layershell::{
    application,
    reexport::{
        Anchor as LayerAnchor,
        KeyboardInteractivity,
        Layer,
    },
    settings::{LayerShellSettings, Settings},
    to_layer_message,
};
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use notify_rust::Notification;
use reqwest;

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Quit,
    TrackUpdate(Option<TrackInfo>),
    AuthUrl(String),
    AuthComplete(Result<(last_fm_rs::AuthToken, String), String>),
    AuthSessionComplete(Result<(String, String), String>),
    SaveConfig,
    OpenAuthUrl(String),
    CompleteAuth,
    NowPlayingSent(Result<(), String>),
    ScrobbleSent(Result<(), String>),
    AppError(String),
    SendNotification(String, String),
    ToggleIgnore(String),
    TrackInfoReceived(Result<crate::lastfm::Track, String>),
    ImageBytesReceived(Result<Vec<u8>, String>),
}

pub struct App {
    config: Config,
    current_track: Option<TrackInfo>,
    lastfm: Option<Arc<LastFm>>,
    auth_token: Option<last_fm_rs::AuthToken>,
    auth_url: Option<String>,
    error_message: Option<String>,
    now_playing_sent: bool,
    scrobble_sent: bool,
    track_start_time: Option<DateTime<Utc>>,
    last_notified_track: Option<String>,
    auth_attempts: u32,
    last_auth_attempt: Option<DateTime<Utc>>,
    track_image_bytes: Option<Vec<u8>>,
}

impl App {
    fn new(config_path: Option<std::path::PathBuf>) -> Self {
        let config = Config::load(config_path).unwrap_or_default();
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
        }
    }
}

pub fn run(config_path: Option<std::path::PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load(config_path.clone()).unwrap_or_default();
    let anchor = match config.ui.position.anchor {
        Anchor::TopLeft => LayerAnchor::Top | LayerAnchor::Left,
        Anchor::TopRight => LayerAnchor::Top | LayerAnchor::Right,
        Anchor::BottomLeft => LayerAnchor::Bottom | LayerAnchor::Left,
        Anchor::BottomRight => LayerAnchor::Bottom | LayerAnchor::Right,
    };

    let margin = match config.ui.position.anchor {
        Anchor::TopLeft => (config.ui.position.y, config.ui.position.x, 0, 0),
        Anchor::TopRight => (config.ui.position.y, 0, 0, config.ui.position.x),
        Anchor::BottomLeft => (0, config.ui.position.x, config.ui.position.y, 0),
        Anchor::BottomRight => (0, 0, config.ui.position.y, config.ui.position.x),
    };

    let layer_settings = LayerShellSettings {
        anchor,
        layer: Layer::Overlay,
        exclusive_zone: -1,
        size: Some((450, 250)),
        margin,
        keyboard_interactivity: KeyboardInteractivity::OnDemand,
        ..Default::default()
    };

    application(
        move || App::new(config_path.clone()),
        "traac",
        update,
        view,
    )
    .subscription(|state| iced::time::every(Duration::from_secs(state.config.general.poll_interval_secs)).map(|_| Message::Tick))
    .settings(Settings {
        layer_settings,
        ..Default::default()
    })
    .run()?;

    Ok(())
}

fn update(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
                if state.auth_token.is_some() && state.config.lastfm.session_key.is_none() {
                let now = Utc::now();
                let should_attempt = match state.last_auth_attempt {
                    Some(last_attempt) => {
                        (now - last_attempt).num_seconds() >= 3
                    }
                    None => true,
                };

                if should_attempt && state.auth_attempts < 20 {
                    state.last_auth_attempt = Some(now);
                    state.auth_attempts += 1;
                    if let Some(lastfm) = state.lastfm.clone() {
                        if let Some(token) = state.auth_token.clone() {
                            return Task::perform(
                                complete_authentication(lastfm, token),
                                |result| match result {
                                    Ok((session_key, username)) => Message::AuthSessionComplete(Ok((session_key, username))),
                                    Err(e) => Message::AuthSessionComplete(Err(e)),
                                }
                            );
                        }
                    }
                }
            }

            if state.config.lastfm.session_key.is_none() && state.lastfm.is_none() && state.auth_url.is_none() {
                if !state.config.lastfm.api_key.is_empty() && !state.config.lastfm.api_secret.is_empty() {
                    let lastfm = LastFm::new(
                        state.config.lastfm.api_key.clone(),
                        state.config.lastfm.api_secret.clone()
                    );
                    let lastfm_arc = Arc::new(lastfm);
                    state.lastfm = Some(lastfm_arc.clone());
                    return Task::perform(
                        get_auth_token(lastfm_arc),
                        |result| match result {
                            Ok((token, url)) => Message::AuthComplete(Ok((token, url))),
                            Err(e) => Message::AuthComplete(Err(e)),
                        }
                    );
                }
            }

            if state.config.lastfm.session_key.is_some() && state.lastfm.is_none() {
                if !state.config.lastfm.api_key.is_empty() && !state.config.lastfm.api_secret.is_empty() {
                    let mut lfm = LastFm::new(
                        state.config.lastfm.api_key.clone(),
                        state.config.lastfm.api_secret.clone()
                    );
                    if let Some(session_key) = &state.config.lastfm.session_key {
                        lfm = lfm.with_session_key(session_key.clone());
                        state.lastfm = Some(Arc::new(lfm));
                    }
                }
            }
            
            if let Ok(player) = find_player_with_ignore(&state.config.general.ignored_players) {
                let track = get_current_track(&player);
                let is_playing_flag = is_playing(&player);

                if let Some(track) = track {
                    let track_changed = match &state.current_track {
                        Some(current) => {
                            current.artist != track.artist ||
                            current.title != track.title ||
                            current.album != track.album
                        }
                        None => true,
                    };

                    if track_changed {
                        state.current_track = Some(track.clone());
                        state.now_playing_sent = false;
                        state.scrobble_sent = false;
                        state.track_start_time = Some(Utc::now());
                        state.track_image_bytes = None;

                        let mut tasks = Vec::new();

                        if state.config.ui.show_notifications {
                            let notification_key = format!("{} - {}", track.artist, track.title);
                            if state.last_notified_track.as_ref().map_or(true, |last| last != &notification_key) {
                                state.last_notified_track = Some(notification_key.clone());
                                tasks.push(Task::perform(
                                    send_notification(track.artist.clone(), track.title.clone(), track.album.clone()),
                                    |_| Message::Tick
                                ));
                            }
                        }

                        if let Some(lastfm) = state.lastfm.clone() {
                            tasks.push(Task::perform(
                                send_now_playing(lastfm.clone(), track.clone()),
                                Message::NowPlayingSent
                            ));
                            tasks.push(Task::perform(
                                fetch_track_info(lastfm, track),
                                Message::TrackInfoReceived
                            ));
                        }

                        if !tasks.is_empty() {
                            return Task::batch(tasks);
                        }
                    } else if is_playing_flag {
                        if let (Some(start_time), Some(track_duration)) = (state.track_start_time, track.duration) {
                            let elapsed = Utc::now().signed_duration_since(start_time).num_seconds() as u64;
                            let track_duration_secs = track_duration;

                            eprintln!("Scrobble check: elapsed={}, duration={}, scrobble_sent={}, enabled={}", 
                                elapsed, track_duration_secs, state.scrobble_sent, state.config.general.scrobble_enabled);

                            if !state.scrobble_sent && state.config.general.scrobble_enabled {
                                // Last.fm rules: scrobble at 50% duration OR 240s (4 min), whichever is LESS
                                // For tracks < 8 min: use 50%
                                // For tracks >= 8 min: cap at 240s
                                let scrobble_threshold = if track_duration_secs > 0 {
                                    let half_duration = track_duration_secs / 2;
                                    let max_threshold = 240; // 4 minutes cap
                                    std::cmp::min(half_duration, max_threshold)
                                } else {
                                    240 // fallback to 240s if no duration
                                };

                                let should_scrobble = elapsed >= scrobble_threshold;

                                eprintln!("Should scrobble: {} (elapsed={}, threshold={})", 
                                    should_scrobble, elapsed, scrobble_threshold);

                                if should_scrobble && state.lastfm.is_some() {
                                    state.scrobble_sent = true;
                                    eprintln!("Sending scrobble: {} - {}", track.artist, track.title);
                                    if let Some(lastfm) = state.lastfm.clone() {
                                        return Task::perform(
                                            send_scrobble(lastfm, track),
                                            Message::ScrobbleSent
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Task::none()
        }
        Message::TrackUpdate(track) => {
            state.current_track = track;
            Task::none()
        }
        Message::AuthUrl(url) => {
            state.auth_url = Some(url);
            Task::none()
        }
Message::AuthComplete(result) => {
    match result {
        Ok((token, url)) => {
            state.auth_token = Some(token);
            state.auth_url = Some(url);
        }
        Err(e) => {
            state.error_message = Some(e);
        }
    }
    Task::none()
}
        Message::SaveConfig => {
            let _ = state.config.save();
            Task::none()
        }
Message::OpenAuthUrl(url) => {
    let _ = open::that(&url);
    Task::none()
}
Message::CompleteAuth => {
    if let (Some(token), Some(_)) = (&state.auth_token, &state.auth_url) {
        if let Some(lastfm) = state.lastfm.clone() {
            let token_clone = token.clone();
            return Task::perform(
                complete_authentication(lastfm, token_clone),
                |result| match result {
                    Ok((session_key, username)) => Message::AuthSessionComplete(Ok((session_key, username))),
                    Err(e) => Message::AuthSessionComplete(Err(e)),
                }
            );
        }
    }
    Task::none()
}
Message::AuthSessionComplete(result) => {
    match result {
        Ok((session_key, username)) => {
            eprintln!("Auth successful! Username: {}, Session: {}", username, session_key);
            state.config.lastfm.session_key = Some(session_key.clone());
            state.config.lastfm.username = Some(username.clone());

            let mut lfm = LastFm::new(
                state.config.lastfm.api_key.clone(),
                state.config.lastfm.api_secret.clone()
            );
            lfm = lfm.with_session_key(session_key.clone());
            state.lastfm = Some(Arc::new(lfm));

            match state.config.save() {                Ok(_) => {
                    eprintln!("Config saved successfully");
                    state.error_message = None;
                    state.auth_token = None;
                    state.auth_url = None;
                }
                Err(e) => {
                    eprintln!("Failed to save config: {}", e);
                    state.error_message = Some(format!("Failed to save config: {}", e));
                }
            }
        }
        Err(e) => {
            eprintln!("Auth error: {}", e);
            state.error_message = Some(format!("Auth failed: {}", e));
        }
    }
    Task::none()
}
        Message::NowPlayingSent(result) => {
            match result {
                Ok(_) => state.now_playing_sent = true,
                Err(e) => state.error_message = Some(e),
            }
            Task::none()
        }
        Message::ScrobbleSent(result) => {
            match result {
                Ok(_) => state.scrobble_sent = true,
                Err(e) => state.error_message = Some(e),
            }
            Task::none()
        }
        Message::AppError(msg) => {
            state.error_message = Some(msg);
            Task::none()
        }
        Message::ToggleIgnore(player_identity) => {
            if let Some(pos) = state.config.general.ignored_players.iter().position(|p| p == &player_identity) {
                state.config.general.ignored_players.remove(pos);
            } else {
                state.config.general.ignored_players.push(player_identity);
            }
            let _ = state.config.save();
            Task::none()
        }
        Message::Quit => iced::exit(),
        Message::TrackInfoReceived(result) => {
            match result {
                Ok(track) => {
                    if let Some(album) = track.album {
                        if let Some(image) = album.image.last() {
                            let url = image.url.clone();
                            if !url.is_empty() {
                                return Task::perform(fetch_image_bytes(url), Message::ImageBytesReceived);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Track info error: {}", e);
                }
            }
            Task::none()
        }
        Message::ImageBytesReceived(result) => {
            match result {
                Ok(bytes) => {
                    state.track_image_bytes = Some(bytes);
                }
                Err(e) => {
                    eprintln!("Image fetch error: {}", e);
                }
            }
            Task::none()
        }
        _ => Task::none(),
    }
}

async fn fetch_track_info(lastfm: Arc<LastFm>, track: TrackInfo) -> Result<crate::lastfm::Track, String> {
    lastfm.get_track_info(&track.artist, &track.title)
        .await
        .map_err(|e| format!("Track info error: {}", e))
}

async fn fetch_image_bytes(url: String) -> Result<Vec<u8>, String> {
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    Ok(bytes.to_vec())
}

async fn send_now_playing(lastfm: Arc<LastFm>, track: TrackInfo) -> Result<(), String> {
    lastfm.update_now_playing(&track.artist, &track.title, track.album.as_deref())
        .await
        .map_err(|e| format!("Now playing error: {}", e))
}

async fn send_scrobble(lastfm: Arc<LastFm>, track: TrackInfo) -> Result<(), String> {
    lastfm.scrobble(&track.artist, &track.title, track.album.as_deref())
        .await
        .map_err(|e| format!("Scrobble error: {}", e))
}

async fn send_notification(artist: String, title: String, _album: Option<String>) -> Result<(), String> {
    let mut notification = Notification::new();
    notification.summary("Now Playing");
    notification.body(&format!("{} - {}", artist, title));
    notification.appname("traac");

    let _ = notification.show();
    Ok(())
}

async fn get_auth_token(lastfm: Arc<LastFm>) -> Result<(last_fm_rs::AuthToken, String), String> {
    let token = lastfm.get_token().await.map_err(|e| format!("get_token error: {}", e))?;
    let url = lastfm.get_auth_url(&token).map_err(|e| format!("get_auth_url error: {}", e))?;
    eprintln!("Auth token obtained, URL: {}", url);
    Ok((token, url))
}

async fn complete_authentication(lastfm: Arc<LastFm>, token: last_fm_rs::AuthToken) -> Result<(String, String), String> {
    eprintln!("Attempting to get session with token...");
    let session = lastfm.get_session(&token).await.map_err(|e| {
        eprintln!("get_session error: {:?}", e);
        format!("{:?}", e)
    })?;
    eprintln!("Session obtained: {}", session.key);
    let session_key = session.key;
    let username = session.name;
    Ok((session_key, username))
}

fn view(state: &App) -> Element<'_, Message> {
    let colors = &state.config.ui.color_scheme;
    let base_color: Color = colors.base.parse().unwrap_or(Color::BLACK);
    let _slightly_lighter: Color = colors.slightly_lighter.parse().unwrap_or(Color::from_rgb(0.2, 0.2, 0.3));
    let accent_grey: Color = colors.accent_grey.parse().unwrap_or(Color::from_rgb(0.4, 0.4, 0.5));
    let bright: Color = colors.bright.parse().unwrap_or(Color::WHITE);
    let text_color: Color = colors.text.parse().unwrap_or(Color::WHITE);

    let mut content = column![].spacing(4);

    if let Some(track) = &state.current_track {
        let mut track_info = column![
            text(&track.artist).size(18).color(text_color),
            text(&track.title).size(16).color(bright),
        ].spacing(2);

        if let Some(album) = &track.album {
            track_info = track_info.push(text(album).size(14).color(accent_grey));
        }

        let mut track_row = row![].spacing(10);
        
        if let Some(bytes) = &state.track_image_bytes {
            track_row = track_row.push(
                image(iced::widget::image::Handle::from_bytes(bytes.clone()))
                    .width(Length::Fixed(60.0))
                    .height(Length::Fixed(60.0))
            );
        } else if let Some(art_url) = &track.art_url {
            if art_url.starts_with("file://") {
                let path = art_url.trim_start_matches("file://");
                track_row = track_row.push(
                    image(path)
                        .width(Length::Fixed(60.0))
                        .height(Length::Fixed(60.0))
                );
            }
        }
        
        track_row = track_row.push(track_info);
        content = content.push(track_row);

        if let Ok(player) = find_player_with_ignore(&[]) {
            let identity = player.identity().to_string();
            let is_ignored = state.config.general.ignored_players.contains(&identity);
            
            content = content.push(
                button(text(if is_ignored { "Unignore Player" } else { "Ignore Player" }))
                    .on_press(Message::ToggleIgnore(identity))
                    .padding(5)
            );
        }

        if state.now_playing_sent {
            content = content.push(
                text("Now Playing sent")
                    .size(12)
                    .color(Color::from_rgb(0.5, 0.8, 0.5))
            );
        }

        if state.scrobble_sent {
            content = content.push(
                text("Scrobbled")
                    .size(12)
                    .color(Color::from_rgb(0.5, 0.8, 0.5))
            );
        }
    } else {
        content = content.push(text("No track playing").size(14).color(accent_grey));
    }

    if let Some(error) = &state.error_message {
        content = content.push(
            text(format!("Error: {}", error))
                .size(12)
                .color(Color::from_rgb(1.0, 0.3, 0.3))
        );
    }

if state.config.lastfm.session_key.is_none() {
            if state.config.lastfm.api_key.is_empty() || state.config.lastfm.api_secret.is_empty() {
                content = content.push(
                    text("Missing Last.fm API credentials").size(12).color(Color::from_rgb(1.0, 0.3, 0.3))
                );
                content = content.push(
                    text("Add api_key and api_secret to config.toml").size(10).color(accent_grey)
                );
            } else if state.auth_url.is_none() {
                content = content.push(
                    text("Getting authorization token...").size(12).color(accent_grey)
                );
            } else if state.auth_token.is_some() {
                content = content
                .push(text("Authorization in progress").size(12).color(bright));
              
              if let Some(url) = &state.auth_url {
                let attempt_msg = if state.auth_attempts > 0 {
                    format!("Attempting auth ({}/20)...", state.auth_attempts)
                } else {
                    "Waiting for authorization...".to_string()
                };

                content = content
                .push(
                  button(text("Open Authorization Page"))
                  .on_press(Message::OpenAuthUrl(url.clone()))
                )
                .push(
                  text("1. Authorize the app in your browser").size(10).color(accent_grey)
                )
                .push(
                  text(attempt_msg).size(10).color(accent_grey)
                )
                .push(
                  button(text("Complete Manually"))
                  .on_press(Message::CompleteAuth)
                );
              }
            } else {
                content = content.push(
                    text("Waiting for authorization...").size(12).color(accent_grey)
                );
            }
        }

    container(
        content
    )
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_theme| container::Style {
        background: Some(base_color.into()),
        ..Default::default()
    })
    .into()
}
