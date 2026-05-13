use iced::widget::{column, container, text, button, row, image};
use iced::{Color, Element, Length};
use crate::ui::state::App;
use crate::ui::types::Message;

pub fn view(state: &App) -> Element<'_, Message> {
    if !state.visible {
        return container(column![])
            .width(0)
            .height(0)
            .into();
    }
    let colors = &state.parsed_colors;
    let base_color = colors.base;
    let accent_grey = colors.accent_grey;
    let bright = colors.bright;
    let text_color = colors.text;

    let mut content = column![].spacing(8);

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

        if state.config.general.scrobble_sanity_check {
            if state.track_verified {
                content = content.push(
                    text("Verified on Last.fm")
                        .size(10)
                        .color(Color::from_rgb(0.5, 0.8, 0.5))
                );
            } else {
                content = content.push(
                    text("Track verification pending/failed")
                        .size(10)
                        .color(accent_grey)
                );
            }
        }
    } else {
        content = content.push(text("No track playing").size(14).color(accent_grey));
    }

    // Players list
    if !state.all_players.is_empty() {
        let mut players_col = column![text("Players:").size(12).color(accent_grey)].spacing(4);
        for player in &state.all_players {
            let is_ignored = state.config.general.ignored_players.contains(player);
            let player_row = row![
                text(player).size(12).color(if is_ignored { accent_grey } else { text_color }),
                button(text(if is_ignored { "Unignore" } else { "Ignore" }))
                    .on_press(Message::ToggleIgnore(player.clone()))
                    .padding(2)
            ].spacing(8);
            players_col = players_col.push(player_row);
        }
        content = content.push(players_col);
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
