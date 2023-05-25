use shared_utils::{decode_header, decode_msg, MSG_SIZE_BYTES, Msg};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use iced_futures::futures::{
    channel::mpsc::{self},
    StreamExt,
};
use iced_futures::futures::sink::SinkExt;
use iced_native::subscription::{self, Subscription};

#[derive(Debug, Clone )]
pub enum Event {
    FailConnection,
    Connected(mpsc::Sender<Input>),
    MsgRecived(Msg),
}

pub enum Input {
    MsgCreated,
}

pub enum State {
    Disconnected,
    Connected(mpsc::Receiver<Input>, TcpStream),
}

pub fn connect() -> Subscription<Event> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let mut state = State::Disconnected;
            let mut incoming_msg_len_buf = vec![0; MSG_SIZE_BYTES];

            loop {
                match &mut state {
                    State::Disconnected => {
                        match TcpStream::connect("127.0.0.1:8000").await {
                            Ok(socket) => {
                                let (tx, rx) = mpsc::channel(100);
                                let _ = output.send(Event::Connected(tx)).await;
                                state = State::Connected(rx, socket);
                            }
                            Err(_) => {
                                tokio::time::sleep(
                                    tokio::time::Duration::from_secs(1),
                                )
                                .await;
                                let _ = output.send(Event::FailConnection).await;
                            }
                        }
                    }
                    State::Connected(rx, socket) => {
                        let (mut reader, mut writer) = socket.split();

                        tokio::select! {
                            bytes_readed = reader.read(&mut incoming_msg_len_buf) => {
                                if let Ok(bytes_readed) = bytes_readed {
                                    if bytes_readed != 0 {
                                        let incoming_msg_len = decode_header(&incoming_msg_len_buf[..]) as usize;
                                        let mut buf = vec![0; incoming_msg_len];
                                        reader.read(&mut buf).await.unwrap();

                                        let recived_msg = decode_msg(&buf).unwrap();
                                        let _ = output.send(Event::MsgRecived(recived_msg)).await;
                                        continue;
                                    }
                                }
                                let _ = output.send(Event::FailConnection).await;
                                state = State::Disconnected;
                            }
                            msg = rx.select_next_some() => {
                                match msg {
                                    Input::MsgCreated => {
                                        if writer.write_all("".as_bytes()).await.is_err() {
                                            let _ = output.send(Event::FailConnection).await;
                                            state = State::Disconnected;
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}