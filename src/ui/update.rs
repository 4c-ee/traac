use iced::Task;
use std::sync::Arc;
use chrono::Utc;
use notify_rust::Notification;
use crate::ui::state::App;
use crate::ui::types::Message;
use crate::mpris::TrackInfo;
use crate::lastfm::LastFm;
use crate::error::TraacError;

pub fn update(state: &mut App, message: Message) -> Task<Message> {
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
                                    Err(e) => Message::AuthSessionComplete(Err(e.to_string())),
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
                            Err(e) => Message::AuthComplete(Err(e.to_string())),
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
            
            // Scrobble check
            if let Some(track) = &state.current_track {
                if !state.scrobble_sent && state.config.general.scrobble_enabled {
                    let now = Utc::now();
                    let elapsed = state.track_total_played_secs + 
                        state.last_resume_time.map_or(0, |last| now.signed_duration_since(last).num_seconds() as u64);

                    if let Some(track_duration_secs) = track.duration {
                        // Sanity check: only scrobble if verified (if sanity check enabled)
                        let can_scrobble = if state.config.general.scrobble_sanity_check {
                            state.track_verified
                        } else {
                            true
                        };

                        if can_scrobble {
                            // Last.fm rules: scrobble at 50% duration OR 240s (4 min), whichever is LESS
                            let scrobble_threshold = {
                                let half_duration = track_duration_secs / 2;
                                let max_threshold = 240; // 4 minutes cap
                                std::cmp::min(half_duration, max_threshold)
                            };

                            if elapsed >= scrobble_threshold && state.lastfm.is_some() {
                                state.scrobble_sent = true;
                                if let Some(lastfm) = state.lastfm.clone() {
                                    let track_clone = track.clone();
                                    return Task::perform(
                                        send_scrobble(lastfm, track_clone),
                                        |res| Message::ScrobbleSent(res.map_err(|e| e.to_string()))
                                    );
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
        Message::MprisTrackChanged(track) => {
            handle_track_change(state, track)
        }
        Message::MprisStatusChanged(is_playing) => {
            let now = Utc::now();
            if is_playing {
                if state.last_resume_time.is_none() {
                    state.last_resume_time = Some(now);
                }
            } else if let Some(last) = state.last_resume_time {
                state.track_total_played_secs += now.signed_duration_since(last).num_seconds() as u64;
                state.last_resume_time = None;
            }
            Task::none()
        }
        Message::MprisStopped => {
            state.current_track = None;
            state.track_image_bytes = None;
            state.track_total_played_secs = 0;
            state.last_resume_time = None;
            state.now_playing_sent = false;
            state.scrobble_sent = false;
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
            let _ = state.config.save(state.config_path.clone());
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
                            Err(e) => Message::AuthSessionComplete(Err(e.to_string())),
                        }
                    );
                }
            }
            Task::none()
        }
        Message::AuthSessionComplete(result) => {
            match result {
                Ok((session_key, username)) => {
                    state.config.lastfm.session_key = Some(session_key.clone());
                    state.config.lastfm.username = Some(username.clone());

                    let mut lfm = LastFm::new(
                        state.config.lastfm.api_key.clone(),
                        state.config.lastfm.api_secret.clone()
                    );
                    lfm = lfm.with_session_key(session_key.clone());
                    state.lastfm = Some(Arc::new(lfm));

                    match state.config.save(state.config_path.clone()) {
                        Ok(_) => {
                            state.error_message = None;
                            state.auth_token = None;
                            state.auth_url = None;
                        }
                        Err(e) => {
                            state.error_message = Some(format!("Failed to save config: {}", e));
                        }
                    }
                }
                Err(e) => {
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
            let _ = state.config.save(state.config_path.clone());
            Task::none()
        }
        Message::Quit => iced::exit(),
        Message::ToggleWindow => {
            state.visible = !state.visible;
            Task::none()
        }
        Message::TrayEvent(_event) => {
            Task::none()
        }
        Message::TrayMenuEvent(_event) => {
            Task::none()
        }
        Message::NoOp => Task::none(),
        Message::TrackInfoReceived(result) => {
            match result {
                Ok(track) => {
                    state.track_verified = true;
                    if let Some(album) = track.album {
                        if let Some(image) = album.image.last() {
                            let url = image.url.clone();
                            if !url.is_empty() {
                                if let Some(bytes) = state.image_cache.get(&url) {
                                    state.track_image_bytes = Some(bytes.clone());
                                } else {
                                    return Task::perform(
                                        fetch_image_bytes(url), 
                                        |res| Message::ImageBytesReceived(res.map_err(|e| e.to_string()))
                                    );
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    state.track_verified = false;
                    eprintln!("Track info error: {}", e);
                }
            }
            Task::none()
        }
        Message::ImageBytesReceived(result) => {
            match result {
                Ok((url, bytes)) => {
                    state.image_cache.insert(url, bytes.clone());
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

fn handle_track_change(state: &mut App, track: TrackInfo) -> Task<Message> {
    state.current_track = Some(track.clone());
    state.now_playing_sent = false;
    state.scrobble_sent = false;
    state.track_start_time = Some(Utc::now());
    state.track_image_bytes = None;
    state.track_verified = false;
    state.track_total_played_secs = 0;
    state.last_resume_time = Some(Utc::now());

    let mut tasks = Vec::new();

    if let Some(art_url) = &track.art_url {
        if let Some(bytes) = state.image_cache.get(art_url) {
            state.track_image_bytes = Some(bytes.clone());
        }
    }

    if state.config.ui.show_notifications {
        let notification_key = format!("{} - {}", track.artist, track.title);
        if state.last_notified_track.as_ref().map_or(true, |last| last != &notification_key) {
            state.last_notified_track = Some(notification_key.clone());

            let art_path = if let Some(url) = &track.art_url {
                if url.starts_with("file://") {
                    Some(url.trim_start_matches("file://").to_string())
                } else {
                    None
                }
            } else {
                None
            };

            tasks.push(Task::perform(
                send_notification(track.artist.clone(), track.title.clone(), track.album.clone(), art_path),
                |res| match res {
                    Ok(_) => Message::NoOp,
                    Err(e) => Message::AppError(e.to_string()),
                }
            ));
        }
    }

    if let Some(lastfm) = state.lastfm.clone() {
        let track_clone1 = track.clone();
        tasks.push(Task::perform(
            send_now_playing(lastfm.clone(), track_clone1),
            |res| Message::NowPlayingSent(res.map_err(|e| e.to_string()))
        ));
        let track_clone2 = track.clone();
        tasks.push(Task::perform(
            fetch_track_info(lastfm, track_clone2),
            |res| Message::TrackInfoReceived(res.map_err(|e| e.to_string()))
        ));
    }

    if !tasks.is_empty() {
        Task::batch(tasks)
    } else {
        Task::none()
    }
}

async fn fetch_track_info(lastfm: Arc<LastFm>, track: TrackInfo) -> Result<crate::lastfm::Track, TraacError> {
    lastfm.get_track_info(&track.artist, &track.title).await
}

async fn fetch_image_bytes(url: String) -> Result<(String, Vec<u8>), TraacError> {
    let response = reqwest::get(&url).await?;
    let bytes = response.bytes().await?;
    Ok((url, bytes.to_vec()))
}

async fn send_now_playing(lastfm: Arc<LastFm>, track: TrackInfo) -> Result<(), TraacError> {
    lastfm.update_now_playing(&track.artist, &track.title, track.album.as_deref())
        .await
        .map_err(|e| TraacError::LastFmApi(e))
}

async fn send_scrobble(lastfm: Arc<LastFm>, track: TrackInfo) -> Result<(), TraacError> {
    lastfm.scrobble(&track.artist, &track.title, track.album.as_deref())
        .await
        .map_err(|e| TraacError::LastFmApi(e))
}

async fn send_notification(artist: String, title: String, _album: Option<String>, image_path: Option<String>) -> Result<(), TraacError> {
    let mut notification = Notification::new();
    notification.summary("Now Playing");
    notification.body(&format!("{} - {}", artist, title));
    notification.appname("traac");
    if let Some(path) = image_path {
        notification.icon(&path);
    }

    notification.show().map_err(|e| TraacError::Other(e.to_string()))?;
    Ok(())
}

async fn get_auth_token(lastfm: Arc<LastFm>) -> Result<(last_fm_rs::AuthToken, String), TraacError> {
    let token = lastfm.get_token().await?;
    let url = lastfm.get_auth_url(&token)?;
    Ok((token, url))
}

async fn complete_authentication(lastfm: Arc<LastFm>, token: last_fm_rs::AuthToken) -> Result<(String, String), TraacError> {
    let session = lastfm.get_session(&token).await?;
    let session_key = session.key;
    let username = session.name;
    Ok((session_key, username))
}
