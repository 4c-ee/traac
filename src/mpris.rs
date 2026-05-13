#![allow(dead_code)]

use mpris::{Metadata, Player, PlayerFinder};
pub use mpris::{PlaybackStatus, Event};

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

use crate::error::{TraacError, Result as TraacResult};

pub fn find_player() -> TraacResult<Player> {
    find_player_with_ignore(&[])
}

pub fn find_player_with_ignore(ignored: &[String]) -> TraacResult<Player> {
    let finder = PlayerFinder::new()?;
    let all_players = finder
        .find_all()?;
    
    for player in all_players {
        if !is_player_ignored(player.identity(), player.bus_name(), ignored) {
            return Ok(player);
        }
    }
    
    Err(TraacError::NoPlayerFound)
}

fn is_player_ignored(identity: &str, bus_name: &str, ignored: &[String]) -> bool {
    let bus_name_short = bus_name.strip_prefix("org.mpris.MediaPlayer2.").unwrap_or(bus_name);
    
    ignored.iter().any(|pattern| {
        if let Ok(p) = glob::Pattern::new(pattern) {
            let options = glob::MatchOptions {
                case_sensitive: false,
                ..Default::default()
            };
            p.matches_with(identity, options) || 
            p.matches_with(bus_name, options) || 
            p.matches_with(bus_name_short, options)
        } else {
            false
        }
    })
}

pub fn list_all_players() -> TraacResult<Vec<String>> {
    let finder = PlayerFinder::new()?;
    let all_players = finder
        .find_all()?;
    
    Ok(all_players.iter().map(|p| p.identity().to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_player_ignored() {
        let ignored = vec!["vlc".to_string(), "firefox.*".to_string(), "Spotify".to_string()];
        
        // VLC - matches bus name short
        assert!(is_player_ignored("VLC media player", "org.mpris.MediaPlayer2.vlc", &ignored));
        
        // Firefox - matches bus name short with glob
        assert!(is_player_ignored("Firefox", "org.mpris.MediaPlayer2.firefox.instance123", &ignored));
        
        // Spotify - matches identity (case-insensitive)
        assert!(is_player_ignored("spotify", "org.mpris.MediaPlayer2.spotify.other", &ignored));
        
        // Something else - not ignored
        assert!(!is_player_ignored("Chromium", "org.mpris.MediaPlayer2.chromium", &ignored));
    }
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
