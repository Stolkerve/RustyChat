mod client;

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{executor, Application, Command, Length, Settings, Theme};
use iced_futures::futures::channel::mpsc;

fn main() -> Result<(), iced::Error> {
    RustyChat::run(Settings::default())
}

struct RustyChat {
    messages: Vec<client::Input>,
    new_message: String,
    sender: Option<mpsc::Sender<client::Input>>,
    disconected: bool,
}

#[derive(Debug, Clone)]
enum Messages {
    Subscription(client::Event),
}

impl Application for RustyChat {
    type Executor = executor::Default;
    type Message = Messages;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                messages: Vec::new(),
                new_message: String::from(""),
                sender: None,
                disconected: true,
            },
            Command::none(),
        )
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        client::connect().map(Messages::Subscription)
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Messages::Subscription(event) => match event {
                client::Event::FailConnection => {
                    self.disconected = true;
                    return Command::none();
                }
                client::Event::Connected(sender) => {
                    self.sender = Some(sender);
                    self.disconected = false;
                    return Command::none();
                }
                client::Event::MsgRecived(_) => {
                    return Command::none();
                }
            },
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        if !self.disconected {
            return container(
                column![
                    scrollable(text("Hola"))
                        .width(Length::Fill)
                        .height(Length::Fill),
                    row![text_input("", ""), button("Enviar")]
                ]
                .padding(20),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }
        container(text("Disconnected...")).into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn title(&self) -> String {
        String::from("RustyChat")
    }
}
