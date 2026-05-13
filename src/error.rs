use thiserror::Error;

#[derive(Error, Debug)]
pub enum TraacError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config parse error: {0}")]
    Config(#[from] toml::de::Error),

    #[error("Config serialize error: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),

    #[error("Last.fm API error: {0}")]
    LastFmApi(#[from] last_fm_rs::Error),

    #[error("MPRIS D-Bus error: {0}")]
    MprisDBus(#[from] mpris::DBusError),

    #[error("MPRIS finding error: {0}")]
    MprisFinding(#[from] mpris::FindingError),

    #[error("Tray error: {0}")]
    Tray(#[from] tray_icon::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("GTK error: {0}")]
    Gtk(String),

    #[error("No player found")]
    NoPlayerFound,

    #[error("General error: {0}")]
    Other(String),
}

// Custom Result type
pub type Result<T> = std::result::Result<T, TraacError>;
