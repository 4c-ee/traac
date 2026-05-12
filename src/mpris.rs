#![allow(dead_code)]

use mpris::{Metadata, Player, PlayerFinder, PlaybackStatus};

#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub artist: String,
    pub title: String,
    pub album: Option<String>,
    pub duration: Option<u64>,
    pub art_url: Option<String>,
}

impl TrackInfo {
    pub fn from_metadata(metadata: &Metadata) -> Option<Self> {
        let artist = metadata
            .artists()
            .and_then(|v| v.first().cloned())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown Artist".to_string());

        let title = metadata
            .title()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let album = metadata.album_name().map(|s| s.to_string());

    let duration = metadata.length_in_microseconds().map(|us| us / 1_000_000);

    let art_url = metadata.art_url().map(|s| s.to_string());

    Some(Self {
      artist,
      title,
      album,
      duration,
      art_url,
    })
    }
}

pub fn find_player() -> Result<Player, String> {
    find_player_with_ignore(&[])
}

pub fn find_player_with_ignore(ignored: &[String]) -> Result<Player, String> {
    let finder = PlayerFinder::new().map_err(|e| format!("D-Bus error: {}", e))?;
    let all_players = finder
        .find_all()
        .map_err(|e| format!("No player found: {}", e))?;
    
    for player in all_players {
        let player_name = player.identity();
        if !ignored.iter().any(|ignored_name| {
            ignored_name.to_lowercase() == player_name.to_lowercase()
        }) {
            return Ok(player);
        }
    }
    
    Err("No non-ignored player found".to_string())
}

pub fn list_all_players() -> Result<Vec<String>, String> {
    let finder = PlayerFinder::new().map_err(|e| format!("D-Bus error: {}", e))?;
    let all_players = finder
        .find_all()
        .map_err(|e| format!("No player found: {}", e))?;
    
    Ok(all_players.iter().map(|p| p.identity().to_string()).collect())
}

pub fn get_current_track(player: &Player) -> Option<TrackInfo> {
    let metadata = player.get_metadata().ok()?;
    TrackInfo::from_metadata(&metadata)
}

pub fn is_playing(player: &Player) -> bool {
    player
        .get_playback_status()
        .ok()
        .map(|s| s == PlaybackStatus::Playing)
        .unwrap_or(false)
}
