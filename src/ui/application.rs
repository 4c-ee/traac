use crate::config::Config;
use iced::{widget::text, Color, Element, Length};
use iced_layershell::{settings::Settings, Application, Shell};

pub struct TraacApp {
    config: Config,
}

#[derive(Debug, Clone)]
pub enum Message {
    Quit,
}

impl Application for TraacApp {
    type Renderer = iced::Renderer;

    type Message = Message;

    fn new(_flags: ()) -> (Self, iced::Command<Self::Message>) {
        let config = Config::load().unwrap_or_default();
        (Self { config }, iced::Command::none())
    }

    fn view(&self) -> Element<Self::Message> {
        let colors = &self.config.ui.color_scheme;
        text("traac - last.fm scrobbler")
            .color(colors.text.parse().unwrap_or(Color::WHITE))
            .into()
    }

    fn title(&self) -> String {
        "traac".to_string()
    }

    fn shell_settings(&self) -> iced_layershell::settings::LayerSettings {
        let pos = &self.config.ui.position;
        let anchor = match pos.anchor {
            crate::config::Anchor::TopLeft => Shell::TopLeft,
            crate::config::Anchor::TopRight => Shell::TopRight,
            crate::config::Anchor::BottomLeft => Shell::BottomLeft,
            crate::config::Anchor::BottomRight => Shell::BottomRight,
        };

        iced_layershell::settings::LayerSettings {
            anchor,
            exclusive_zone: false,
            output_name: Some("traac".to_string()),
            namespace: Some("traac".to_string()),
            size: Some(iced::Size::new(300.0, 100.0)),
            margin: iced::Point::new(pos.x as f32, pos.y as f32),
            ..Default::default()
        }
    }
}

pub fn run_ui() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default();

    TraacApp::run(settings)?;

    Ok(())
}
