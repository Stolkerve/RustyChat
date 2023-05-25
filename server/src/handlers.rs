use std::net::SocketAddr;

use shared_utils::{decode_header, encode_bytes, decode_msg_type, MSG_SIZE_BYTES, MsgType};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::broadcast::{Sender, Receiver},
};

pub fn new_conection(
    mut socket: TcpStream,
    addr: SocketAddr,
    tx: Sender<(String, Vec<u8>)>,
    mut rx: Receiver<(String, Vec<u8>)>
) {
    tokio::spawn(async move {
        let (mut reader, mut writer) = socket.split();
        let mut msg_len_buf = vec![0; MSG_SIZE_BYTES];

        loop {
            tokio::select! {
                bytes_readed = reader.read(&mut msg_len_buf) => {
                    let n = bytes_readed.unwrap();
                    if n == 0 {
                        println!("Peer {:?} disconected", addr);
                        break;
                    }
                    let len = decode_header(&msg_len_buf[..]);

                    let mut buf = vec![0; len as usize];
                    reader.read(&mut buf).await.unwrap();
                    let msg = decode_msg_type(&buf).unwrap();
                    match msg {
                        MsgType::Msg(_) => {
                            tx.send((addr.to_string(), encode_bytes(buf))).unwrap();
                        },
                        MsgType::Login(_) => todo!(),
                        MsgType::Register(_) => todo!(),
                    }
                },
                msg = rx.recv() => {
                    let (sender_addr, msg) = msg.unwrap();
                    if addr.to_string() != sender_addr {
                        writer.write_all(&msg[..]).await.unwrap();
                    }
                }
            }
        }
    });

}