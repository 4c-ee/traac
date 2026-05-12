#![allow(dead_code)]

use mpris::{Metadata, Player, PlayerFinder, PlaybackStatus};

#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub artist: String,
    pub title: String,
    pub album: Option<String>,
    pub duration: Option<u64>,
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

        Some(Self {
            artist,
            title,
            album,
            duration,
        })
    }
}

pub fn find_player() -> Result<Player, String> {
    let finder = PlayerFinder::new().map_err(|e| format!("D-Bus error: {}", e))?;
    finder
        .find_active()
        .or_else(|_| finder.find_first())
        .map_err(|e| format!("No player found: {}", e))
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
