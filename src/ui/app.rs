use crate::config::Config;
use iced::{
    widget::{column, container, text},
    Color, Element, Length, Task,
};
use iced_layershell::{
    application,
    reexport::{
        Anchor,
        KeyboardInteractivity,
        Layer,
    },
    settings::{LayerShellSettings, Settings},
    to_layer_message,
};

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Quit,
}

#[derive(Clone, Default)]
pub struct App {
    config: Config,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load().unwrap_or_default();
    let anchor = match config.ui.position.anchor {
        crate::config::Anchor::TopLeft => Anchor::Top | Anchor::Left,
        crate::config::Anchor::TopRight => Anchor::Top | Anchor::Right,
        crate::config::Anchor::BottomLeft => Anchor::Bottom | Anchor::Left,
        crate::config::Anchor::BottomRight => Anchor::Bottom | Anchor::Right,
    };

    let layer_settings = LayerShellSettings {
        anchor,
        layer: Layer::Overlay,
        exclusive_zone: -1,
        size: Some((300, 100)),
        margin: (
            config.ui.position.y,
            config.ui.position.x,
            config.ui.position.y,
            config.ui.position.x,
        ),
        keyboard_interactivity: KeyboardInteractivity::OnDemand,
        ..Default::default()
    };

    let app = App { config: config.clone() };

    application(
        move || app.clone(),
        "traac",
        update,
        view,
    )
    .settings(Settings {
        layer_settings,
        ..Default::default()
    })
    .run()?;

    Ok(())
}

fn update(_state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Tick => Task::none(),
        Message::Quit => iced::exit(),
        _ => Task::none(),
    }
}

fn view(state: &App) -> Element<'_, Message> {
    let colors = &state.config.ui.color_scheme;
    let base_color: Color = colors.base.parse().unwrap_or(Color::BLACK);
    let text_color: Color = colors.text.parse().unwrap_or(Color::WHITE);

    container(
        column![
            text("traac").size(20).color(text_color),
            text("placeholder").size(12).color(text_color),
        ]
        .spacing(4),
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
