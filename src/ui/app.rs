use crate::config::{Anchor, Config};
use crate::lastfm::LastFm;
use crate::mpris::{find_player, get_current_track, is_playing, TrackInfo};
use iced::widget::{column, container, text, text_input, button};
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

const POLL_INTERVAL: Duration = Duration::from_secs(2);

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Quit,
    TrackUpdate(Option<TrackInfo>),
    AuthToken(String),
    AuthUrl(String),
    AuthComplete(Result<(String, String), String>),
    SaveConfig,
    OpenAuthUrl(String),
    CheckAuth,
    NowPlayingSent,
    ScrobbleSent,
    AppError(String),
    SendNotification(String, String),
}

pub struct App {
    config: Config,
    current_track: Option<TrackInfo>,
    lastfm: Option<Arc<LastFm>>,
    auth_token: Option<String>,
    auth_url: Option<String>,
    auth_input: String,
    error_message: Option<String>,
    now_playing_sent: bool,
    scrobble_sent: bool,
    track_start_time: Option<DateTime<Utc>>,
    last_notified_track: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            config: Config::load().unwrap_or_default(),
            current_track: None,
            lastfm: None,
            auth_token: None,
            auth_url: None,
            auth_input: String::new(),
            error_message: None,
            now_playing_sent: false,
            scrobble_sent: false,
            track_start_time: None,
            last_notified_track: None,
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load().unwrap_or_default();
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
        size: Some((400, 150)),
        margin,
        keyboard_interactivity: KeyboardInteractivity::OnDemand,
        ..Default::default()
    };

    application(
        App::default,
        "traac",
        update,
        view,
    )
    .subscription(|_| iced::time::every(POLL_INTERVAL).map(|_| Message::Tick))
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
            if let Ok(player) = find_player() {
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
                    
                    if state.config.ui.show_notifications {
                        let notification_key = format!("{} - {}", track.artist, track.title);
                        if state.last_notified_track.as_ref().map_or(true, |last| last != &notification_key) {
                            state.last_notified_track = Some(notification_key.clone());
                            return Task::perform(
                                send_notification(track.artist.clone(), track.title.clone(), track.album.clone()),
                                |_| Message::Tick
                            );
                        }
                    }
                    
                    if let Some(lastfm) = state.lastfm.clone() {
                        return Task::perform(
                            send_now_playing(lastfm, track),
                            |_| Message::NowPlayingSent
                        );
                    }
                } else if is_playing_flag {
                        if let (Some(start_time), Some(track_duration)) = (state.track_start_time, track.duration) {
                            let elapsed = Utc::now().signed_duration_since(start_time).num_seconds() as u64;
                            let track_duration_secs = track_duration / 1000;
                            
                            if !state.scrobble_sent && state.config.general.scrobble_enabled {
                                let should_scrobble = elapsed >= 240 || 
                                    (track_duration_secs > 0 && elapsed >= track_duration_secs / 2);
                                    
                                if should_scrobble && state.lastfm.is_some() {
                                    state.scrobble_sent = true;
                                    if let Some(lastfm) = state.lastfm.clone() {
                                        return Task::perform(
                                            send_scrobble(lastfm, track),
                                            |_| Message::ScrobbleSent
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
        Message::AuthToken(token) => {
            state.auth_token = Some(token);
            Task::none()
        }
        Message::AuthUrl(url) => {
            state.auth_url = Some(url);
            Task::none()
        }
        Message::AuthComplete(result) => {
            match result {
                Ok((session_key, username)) => {
                    state.config.lastfm.session_key = Some(session_key);
                    state.config.lastfm.username = Some(username);
                    let _ = state.config.save();
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
        Message::CheckAuth => {
            if state.config.lastfm.session_key.is_some() {
                Task::none()
            } else if !state.config.lastfm.api_key.is_empty() && !state.config.lastfm.api_secret.is_empty() {
                let lastfm = LastFm::new(
                    state.config.lastfm.api_key.clone(), 
                    state.config.lastfm.api_secret.clone()
                );
                state.lastfm = Some(Arc::new(lastfm));
                Task::none()
            } else {
                state.error_message = Some("Missing Last.fm API credentials".to_string());
                Task::none()
            }
        }
        Message::NowPlayingSent => {
            state.now_playing_sent = true;
            Task::none()
        }
        Message::ScrobbleSent => {
            state.scrobble_sent = true;
            Task::none()
        }
        Message::AppError(msg) => {
            state.error_message = Some(msg);
            Task::none()
        }
        Message::Quit => iced::exit(),
        _ => Task::none(),
    }
}

async fn send_now_playing(lastfm: Arc<LastFm>, track: TrackInfo) -> Result<(), String> {
    let _ = lastfm.update_now_playing(&track.artist, &track.title, track.album.as_deref()).await;
    Ok(())
}

async fn send_scrobble(lastfm: Arc<LastFm>, track: TrackInfo) -> Result<(), String> {
    let _ = lastfm.scrobble(&track.artist, &track.title, track.album.as_deref()).await;
    Ok(())
}

async fn send_notification(artist: String, title: String, _album: Option<String>) -> Result<(), String> {
    let mut notification = Notification::new();
    notification.summary("Now Playing");
    notification.body(&format!("{} - {}", artist, title));
    notification.appname("traac");
    
    let _ = notification.show();
    Ok(())
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
        content = content
            .push(text(&track.artist).size(18).color(text_color))
            .push(text(&track.title).size(16).color(bright));

        if let Some(album) = &track.album {
            content = content.push(text(album).size(14).color(accent_grey));
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
        content = content
            .push(text("Last.fm not authenticated").size(12).color(accent_grey));

        if let Some(url) = &state.auth_url {
            content = content
                .push(
                    button(text("Open Auth URL"))
                        .on_press(Message::OpenAuthUrl(url.clone()))
                )
                .push(
                    text_input("Enter token from URL", &state.auth_input)
                        .on_input(Message::AuthToken)
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
