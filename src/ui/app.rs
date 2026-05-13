use crate::config::{Anchor, Config};
use crate::mpris::{find_player_with_ignore, get_current_metadata, is_playing, TrackInfo, Event as MprisEvent};
use crate::error::TraacError;
use iced::Subscription;
use iced_layershell::{
    application,
    reexport::{
        Anchor as LayerAnchor,
        KeyboardInteractivity,
        Layer,
    },
    settings::{LayerShellSettings, Settings},
};
use std::sync::Arc;
use std::time::Duration;
use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem, MenuEvent},
    TrayIconBuilder, TrayIconEvent,
};
use iced::futures::StreamExt;

use crate::ui::state::App;
use crate::ui::types::Message;
use crate::ui::update::update;
use crate::ui::view::view;

pub fn run(config_path: Option<std::path::PathBuf>) -> Result<(), TraacError> {
    #[cfg(target_os = "linux")]
    let _ = gtk::init().map_err(|e| TraacError::Gtk(e.to_string()))?;

    let config = Config::load(config_path.clone()).unwrap_or_default();
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
        size: Some((450, 250)),
        margin,
        keyboard_interactivity: KeyboardInteractivity::OnDemand,
        ..Default::default()
    };

    // Tray icon setup
    let tray_menu = Menu::new();
    let show_hide_item = MenuItem::with_id("show_hide", "Show/Hide", true, None);
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);
    
    let _ = tray_menu.append_items(&[
        &show_hide_item,
        &PredefinedMenuItem::separator(),
        &quit_item,
    ]);

    let icon_path = std::env::current_dir()?.join("logos/traac-single.png");
    let icon = load_icon(&icon_path);

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("traac")
        .with_icon(icon)
        .build()?;
    
    let tray_icon_arc = Arc::new(tray_icon);

    application(
        move || App::new(config_path.clone(), tray_icon_arc.clone()),
        "traac",
        update,
        view,
    )
    .subscription(|state| {
        Subscription::batch(vec![
            iced::time::every(Duration::from_secs(state.config.general.poll_interval_secs)).map(|_| Message::Tick),
            mpris_subscription(state.config.general.ignored_players.clone()),
            tray_subscription(),
        ])
    })
    .settings(Settings {
        layer_settings,
        ..Default::default()
    })
    .run()
    .map_err(|e| TraacError::Other(e.to_string()))?;

    Ok(())
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = ::image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to create icon")
}

fn tray_subscription() -> Subscription<Message> {
    Subscription::run(move || {
        let (mut tx, _rx) = iced::futures::channel::mpsc::channel(100);
        let rx = _rx; // Workaround for ownership
        
        let tray_channel = TrayIconEvent::receiver();
        let menu_channel = MenuEvent::receiver();

        std::thread::spawn(move || {
            loop {
                if let Ok(event) = tray_channel.try_recv() {
                    if let TrayIconEvent::Click { .. } = event {
                        let _ = tx.start_send(Message::ToggleWindow);
                    }
                    let _ = tx.start_send(Message::TrayEvent(event));
                }
                if let Ok(event) = menu_channel.try_recv() {
                    if event.id.0 == "show_hide" {
                        let _ = tx.start_send(Message::ToggleWindow);
                    } else if event.id.0 == "quit" {
                        let _ = tx.start_send(Message::Quit);
                    }
                    let _ = tx.start_send(Message::TrayMenuEvent(event));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        
        rx
    })
}

fn mpris_subscription(ignored_players: Vec<String>) -> Subscription<Message> {
    Subscription::run_with(ignored_players.join(","), |ignored_str| {
        let ignored_players: Vec<String> = ignored_str.split(',').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
        let (mut tx, rx) = iced::futures::channel::mpsc::channel(100);

        std::thread::spawn(move || {
            loop {
                if let Ok(player) = find_player_with_ignore(&ignored_players) {
                    if let Some(metadata) = get_current_metadata(&player) {
                        let playing = is_playing(&player);
                        if tx.start_send(MprisEvent::TrackChanged(metadata)).is_err() {
                            return;
                        }
                        if playing {
                            if tx.start_send(MprisEvent::Playing).is_err() {
                                return;
                            }
                        } else if tx.start_send(MprisEvent::Paused).is_err() {
                            return;
                        }
                    }

                    if let Ok(events) = player.events() {
                        for event in events {
                            if let Ok(event) = event {
                                if tx.start_send(event).is_err() {
                                    return;
                                }
                            }
                        }
                    }

                    let _ = tx.start_send(MprisEvent::PlayerShutDown);
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        });

        rx.map(|event| {
            match event {
                MprisEvent::TrackChanged(metadata) => {
                    if let Some(track) = TrackInfo::from_metadata(&metadata) {
                        Message::MprisTrackChanged(track)
                    } else {
                        Message::NoOp
                    }
                }
                MprisEvent::Playing => {
                    Message::MprisStatusChanged(true)
                }
                MprisEvent::Paused => {
                    Message::MprisStatusChanged(false)
                }
                MprisEvent::Stopped => {
                    Message::MprisStopped
                }
                MprisEvent::PlayerShutDown => {
                    Message::MprisStopped
                }
                _ => Message::NoOp,
            }
        })
    })
}
