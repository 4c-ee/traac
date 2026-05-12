#![allow(dead_code)]

use chrono::Utc;
use last_fm_rs::{Client, NowPlaying, Scrobble};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Track {
    pub name: String,
    pub artist: ArtistInfo,
    pub url: String,
    pub listeners: String,
    pub playcount: String,
    pub duration: Option<u64>,
    pub album: Option<AlbumInfo>,
    pub toptags: Option<TopTags>,
    pub wiki: Option<Wiki>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ArtistInfo {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AlbumInfo {
    pub title: String,
    pub artist: String,
    pub image: Vec<ImageInfo>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ImageInfo {
    #[serde(rename = "#text")]
    pub url: String,
    pub size: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TopTags {
    pub tag: Vec<Tag>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Tag {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Wiki {
    pub summary: String,
    pub content: String,
}

#[derive(Deserialize, Debug, Clone)]
struct TrackResponse {
    track: Track,
}

pub struct LastFm {
    client: Client,
    api_key: String,
}

impl LastFm {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            client: Client::new(api_key.clone(), api_secret),
            api_key,
        }
    }

    pub fn with_session_key(self, session_key: String) -> Self {
        Self {
            client: self.client.with_session_key(session_key),
            api_key: self.api_key,
        }
    }

    pub async fn get_token(&self) -> Result<last_fm_rs::AuthToken, last_fm_rs::Error> {
        self.client.get_token().await
    }

    pub fn get_auth_url(&self, token: &last_fm_rs::AuthToken) -> Result<String, last_fm_rs::Error> {
        self.client.get_auth_url(token)
    }

    pub async fn get_session(&self, token: &last_fm_rs::AuthToken) -> Result<last_fm_rs::SessionKey, last_fm_rs::Error> {
        self.client.get_session(token).await
    }

    pub async fn update_now_playing(&self, artist: &str, track: &str, album: Option<&str>) -> Result<(), last_fm_rs::Error> {
        let mut np = NowPlaying::new(artist, track);
        if let Some(album) = album {
            np = np.with_album(album);
        }
        self.client.update_now_playing(&np).await
    }

    pub async fn scrobble(&self, artist: &str, track: &str, album: Option<&str>) -> Result<(), last_fm_rs::Error> {
        let timestamp = Utc::now().timestamp() as u64;
        let mut scrobble = Scrobble::new(artist, track, timestamp);
        if let Some(album) = album {
            scrobble = scrobble.with_album(album);
        }
        self.client.scrobble(&[scrobble]).await?;
        Ok(())
    }

    pub async fn get_track_info(&self, artist: &str, track: &str) -> Result<Track, String> {
        let url = format!(
            "https://ws.audioscrobbler.com/2.0/?method=track.getInfo&api_key={}&artist={}&track={}&format=json",
            self.api_key,
            urlencoding::encode(artist),
            urlencoding::encode(track)
        );

        let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
        let text = resp.text().await.map_err(|e| e.to_string())?;
        
        let response: TrackResponse = serde_json::from_str(&text).map_err(|e| {
            format!("Failed to parse track info: {}. Response: {}", e, text)
        })?;
        
        Ok(response.track)
    }
}
