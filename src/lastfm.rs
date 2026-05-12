#![allow(dead_code)]

use chrono::Utc;
use last_fm_rs::{Client, NowPlaying, Scrobble};

pub struct LastFm {
    client: Client,
}

impl LastFm {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            client: Client::new(api_key, api_secret),
        }
    }

    pub fn with_session_key(self, session_key: String) -> Self {
        Self {
            client: self.client.with_session_key(session_key),
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
}