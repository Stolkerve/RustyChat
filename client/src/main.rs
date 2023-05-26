mod client;

use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{executor, Application, Command, Length, Settings, Theme, Element};
use iced_futures::futures::channel::mpsc;
use iced_native::color;
use shared_utils::{Msg, MsgDataType};

fn main() -> Result<(), iced::Error> {
    RustyChat::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Messages {
    Subscription(client::Event),
    NewMessageInput(String),
    UsernameInput(String),
    SubmitNewMessage,
    SubmitUsername
}

enum Views {
    UsernameForm,
    Chat
}

struct RustyChat {
    messages: Vec<Msg>,
    new_message_input: String,
    sender: Option<mpsc::Sender<client::Input>>,
    disconected: bool,
    view: Views,
    username: String
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
                new_message_input: String::from(""),
                sender: None,
                disconected: true,
                view: Views::UsernameForm,
                username: String::from("")
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
                    Command::none()
                }
                client::Event::Connected(sender) => {
                    self.sender = Some(sender);
                    self.disconected = false;
                    Command::none()
                }
                client::Event::MsgRecived(msg) => {
                    self.messages.push(msg);
                    Command::none()
                }
            },
            Messages::NewMessageInput(input) => {
                self.new_message_input = input;
                Command::none()
            },
            Messages::SubmitNewMessage => {
                if self.new_message_input.len() == 0 {
                    return Command::none()
                }
                let a = &self.new_message_input;
                if let Some(sender) = &mut self.sender {
                    let msg = Msg {data: MsgDataType::Text(a.to_string()), username: self.username.clone()};
                    sender.start_send(
                        client::Input::MsgCreated(msg.clone())
                    ).unwrap();
                    self.messages.push(msg);
                }
                self.new_message_input.clear();
                Command::none()
            },
            Messages::UsernameInput(username) => {
                self.username = username;
                Command::none()
            },
            Messages::SubmitUsername => {
                if self.username.len() == 0 {
                    return Command::none()
                }
                self.view = Views::Chat;
                Command::none()
            },
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        if !self.disconected {
            match self.view {
                Views::UsernameForm => {
                    let input = text_input("Enter username", &self.username)
                        .on_input(Messages::UsernameInput)
                        .on_submit(Messages::SubmitUsername)
                        .width(350);
                    let button = button("Send")
                        .on_press(Messages::SubmitUsername);
                    return container(
                        column![
                            input, button
                        ].spacing(12)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into();
                },
                Views::Chat => {
                    let input = text_input("", &self.new_message_input)
                        .on_input(Messages::NewMessageInput)
                        .on_submit(Messages::SubmitNewMessage);
                    let submit = button("Send").
                        on_press(Messages::SubmitNewMessage);
                    return container(
                        column![
                            scrollable(
                                Column::with_children(
                                    self.messages.iter().map(|msg| {
                                        let color = if msg.username == self.username { 0xff5c00 } else { 0x005c00 };
                                        match &msg.data {
                                            shared_utils::MsgDataType::Text(msg_text) => {
                                                return row![
                                                    text(format!("[{}]", msg.username)).style(color!(color)),
                                                    text(msg_text),
                                                ].spacing(6)
                                            },
                                        }
                                    }).map(Element::from).collect()
                                ).spacing(6)
                            )
                            .width(Length::Fill)
                            .height(Length::Fill),
                            
                            row![input, submit]
                        ]
                        .spacing(10)
                        .padding(20),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();
                },
            }
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
