use crate::mpris::TrackInfo;
use iced_layershell::to_layer_message;
use tray_icon::{TrayIconEvent, menu::MenuEvent};

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
    ImageBytesReceived(Result<(String, Vec<u8>), String>),
    MprisTrackChanged(TrackInfo),
    MprisStatusChanged(bool),
    NoOp,
    TrayEvent(TrayIconEvent),
    TrayMenuEvent(MenuEvent),
    ToggleWindow,
}
