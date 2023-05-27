mod client;

use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{executor, Application, Command, Element, Length, Settings, Theme};
use iced_futures::futures::channel::mpsc;
use iced_native::color;
use shared_utils::{MsgDataType, ServerMsg, UserMsg, MsgType, LoginMsg};

fn main() -> Result<(), iced::Error> {
    RustyChat::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Views {
    SignupForm,
    LoginForm,
    Chat,
}

#[derive(Debug, Clone)]
enum Messages {
    Subscription(client::Event),
    NewMessageInput(String),
    UsernameInput(String),
    PasswordInput(String),
    ChangeView(Views),
    SubmitNewMessage,
    SubmitSignupForm,
    SubmitLoginForm,
}

struct RustyChat {
    messages: Vec<ServerMsg>,
    new_message_input: String,
    sender: Option<mpsc::Sender<client::Input>>,
    disconected: bool,
    loading: bool,
    view: Views,
    username: String,
    password: String,
    error_msg: String,
    token: String,
}

impl RustyChat {
    pub fn clear(&mut self) {
        self.username.clear();
        self.password.clear();
        self.error_msg.clear();
    }
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
                loading: false,
                view: Views::LoginForm,
                username: String::from(""),
                password: String::from(""),
                error_msg: String::from(""),
                token: String::from(""),
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
                client::Event::ServerRes(res) => {
                    match res {
                        shared_utils::ServerRes::Error(error) => {
                            self.username.clear();
                            self.password.clear();

                            self.error_msg = error;
                        },
                        shared_utils::ServerRes::UserToken(msg) => {
                            self.clear();

                            self.token = msg.token;
                            self.username = msg.username;

                            self.view = Views::Chat;

                        },
                        shared_utils::ServerRes::UserCreated => {
                            self.clear();

                            self.view = Views::LoginForm;
                        },
                    }
                    self.loading = false;
                    Command::none()
                },
            },
            Messages::NewMessageInput(input) => {
                self.new_message_input = input;
                Command::none()
            }
            Messages::SubmitNewMessage => {
                if self.new_message_input.is_empty() {
                    return Command::none();
                }
                let a = &self.new_message_input;
                if let Some(sender) = &mut self.sender {
                    let msg = UserMsg {
                        data: MsgDataType::Text(a.to_string()),
                        username: self.username.clone(),
                        token: self.token.clone(),
                    };
                    sender
                        .start_send(client::Input::MsgType(MsgType::MsgOut(msg.clone())))
                        .unwrap();
                    self.messages.push(ServerMsg {
                        username: msg.username,
                        data: msg.data,
                    });
                }
                self.new_message_input.clear();
                Command::none()
            }
            Messages::UsernameInput(username) => {
                if username.len() <= 30 {
                    self.username = username;
                }
                Command::none()
            }
            Messages::PasswordInput(password) => {
                if password.len() <= 80 {
                    self.password = password;
                }
                Command::none()
            }
            Messages::SubmitSignupForm => {
                if self.username.is_empty() || self.password.is_empty() {
                    self.error_msg = "Username or password cannot be empty".to_string();
                    return Command::none();
                }
                self.loading = true;

                if let Some(sender) = &mut self.sender {
                    let msg = MsgType::Signup(LoginMsg {
                        username: self.username.clone(),
                        password: self.password.clone()
                    });
                    sender
                        .start_send(client::Input::MsgType(msg))
                        .unwrap();
                }
                Command::none()
            }
            Messages::SubmitLoginForm => {
                if self.username.is_empty() || self.password.is_empty() {
                    self.error_msg = "Username or password cannot be empty".to_string();
                    return Command::none();
                }
                self.loading = true;

                if let Some(sender) = &mut self.sender {
                    let msg = MsgType::Login(LoginMsg {
                        username: self.username.clone(),
                        password: self.password.clone()
                    });
                    sender
                        .start_send(client::Input::MsgType(msg))
                        .unwrap();
                }
                Command::none()
            }
            Messages::ChangeView(view) => {
                self.clear();
                self.view = view;
                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        if !self.disconected {
            match self.view {
                Views::SignupForm => {
                    let mut input_name = text_input("Enter username", &self.username)
                        .width(350);
                    let mut input_password = text_input("Enter password", &self.password)
                        .password()
                        .width(350);
                    let mut submit_button = button("Submit");
                    let mut swap_button = button("Login?");

                    if !self.loading {
                        input_name = input_name
                            .on_input(Messages::UsernameInput)
                            .on_submit(Messages::SubmitSignupForm);
                        input_password = input_password
                            .on_input(Messages::PasswordInput)
                            .on_submit(Messages::SubmitSignupForm);
                        submit_button = submit_button.on_press(Messages::SubmitSignupForm);
                        swap_button = swap_button.on_press(Messages::ChangeView(Views::LoginForm));
                    }
                    let row_button = row![submit_button, swap_button].spacing(16);

                    let error_text = text(&self.error_msg).style(color!(0xFB0000));
                    return container(column![text("Signup").size(28), input_name, input_password, row_button, error_text].spacing(12))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x()
                        .center_y()
                        .into();
                },
                Views::LoginForm => {
                    let mut input_name = text_input("Enter username", &self.username)
                        .width(350);
                    let mut input_password = text_input("Enter password", &self.password)
                        .password()
                        .width(350);
                    let mut submit_button = button("Submit");
                    let mut swap_button = button("Signup?");

                    if !self.loading {
                        input_name = input_name
                            .on_input(Messages::UsernameInput)
                            .on_submit(Messages::SubmitLoginForm);
                        input_password = input_password
                            .on_input(Messages::PasswordInput)
                            .on_submit(Messages::SubmitLoginForm);
                        submit_button = submit_button.on_press(Messages::SubmitLoginForm);
                        swap_button = swap_button.on_press(Messages::ChangeView(Views::SignupForm));
                    }
                    let row_button = row![submit_button, swap_button].spacing(16);

                    let error_text = text(&self.error_msg).style(color!(0xFB0000));
                    return container(column![text("Login").size(28), input_name, input_password, row_button, error_text].spacing(12))
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
                    let submit = button("Send").on_press(Messages::SubmitNewMessage);
                    return container(
                        column![
                            scrollable(
                                Column::with_children(
                                    self.messages
                                        .iter()
                                        .map(|msg| {
                                            let color = if msg.username == self.username {
                                                0xff5c00
                                            } else {
                                                0x005c00
                                            };
                                            match &msg.data {
                                                shared_utils::MsgDataType::Text(msg_text) => {
                                                    return row![
                                                        text(format!("[{}]", msg.username))
                                                            .style(color!(color)),
                                                        text(msg_text),
                                                    ]
                                                    .spacing(6)
                                                }
                                            }
                                        })
                                        .map(Element::from)
                                        .collect()
                                )
                                .spacing(6)
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
                }
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
