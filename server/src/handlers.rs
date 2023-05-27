use shared_utils::{
    decode_header, decode_msg_type, encode_msg_type, MsgType, ServerRes,
    MSG_SIZE_BYTES, ServerMsg, TokenMsg,
};
use sqlx::{Pool, Sqlite};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::broadcast::{Receiver, Sender},
};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;
use bcrypt::{DEFAULT_COST, hash, verify};
use std::collections::BTreeMap;

const SECRET: &str = "SECRETO";

#[derive(sqlx::FromRow)]
struct User {
    pub id: i64,
    pub name: String,
    pub password: String,
}

fn verify_jwt(token: String) -> Result<(), jwt::Error> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(SECRET.as_bytes())?;
    let _claims: BTreeMap<String, i64> = token.verify_with_key(&key)?;
    Ok(())
}

pub fn new_conection(
    mut socket: TcpStream,
    addr: String,
    tx: Sender<(String, bool, Vec<u8>)>,
    mut rx: Receiver<(String, bool, Vec<u8>)>,
    db: Pool<Sqlite>,
) {
    tokio::spawn(async move {
        let (mut reader, mut writer) = socket.split();
        let mut msg_len_buf = vec![0; MSG_SIZE_BYTES];
        println!("Peer {:?} conected", addr);

        loop {
            tokio::select! {
                bytes_readed = reader.read(&mut msg_len_buf) => {
                    let n = bytes_readed.unwrap();
                    if n == 0 {
                        println!("Peer {:?} disconected", &addr);
                        break;
                    }
                    let len = decode_header(&msg_len_buf[..]);

                    let mut buf = vec![0; len as usize];
                    reader.read(&mut buf).await.unwrap();
                    let msg = decode_msg_type(&buf).unwrap();
                    match msg {
                        MsgType::MsgOut(msg) => {
                            let peer = reader.peer_addr().unwrap().to_string();
                            match verify_jwt(msg.token.clone()) {
                                Ok(_) => {},
                                Err(err) => {println!("{}", err);}
                            }
                            if verify_jwt(msg.token.clone()).is_ok() {
                                let msg = MsgType::MsgIn(ServerMsg {
                                    username: msg.username,
                                    data: msg.data
                                });
                                tx.send((peer, false, encode_msg_type(&msg))).unwrap();
                                continue;
                            }
                            tx.send((peer, true, encode_msg_type(&MsgType::Server(ServerRes::Error("Msg is not signed.".to_string()))))).unwrap();
                        },
                        MsgType::Login(msg) => {
                            let peer = reader.peer_addr().unwrap().to_string();
                            if reader.peer_addr().unwrap().to_string() != addr {
                                continue;
                            }
                            match sqlx::query_as::<_, User>("SELECT * FROM users WHERE name = ?")
                            .bind(msg.username)
                            .bind(&msg.password)
                            .fetch_one(&db).await {
                                Ok(user) => {
                                    if verify(msg.password, &user.password).unwrap() {
                                        let key: Hmac<Sha256> = Hmac::new_from_slice(SECRET.as_bytes()).unwrap();
                                        let mut claims = BTreeMap::new();
                                        claims.insert("id", user.id);
                                        let token_str = claims.sign_with_key(&key).unwrap();
                                        tx.send((peer, true, encode_msg_type(&MsgType::Server(ServerRes::UserToken(TokenMsg {
                                            token: token_str,
                                            username: user.name
                                        }))))).unwrap();
                                        continue;
                                    }
                                },
                                _ => {},
                            }
                            tx.send((peer, true, encode_msg_type(&MsgType::Server(ServerRes::Error("The username or password are incorrect!.".to_string()))))).unwrap();
                        },
                        MsgType::Signup(msg) => {
                            let peer = reader.peer_addr().unwrap().to_string();
                            if reader.peer_addr().unwrap().to_string() != addr {
                                continue;
                            }
                            let hashed = hash(msg.password, DEFAULT_COST).unwrap();
                            match sqlx::query("INSERT INTO users (name, password) VALUES (?, ?);")
                            .bind(msg.username)
                            .bind(hashed)
                            .execute(&db).await {
                                Ok(_) => {
                                    tx.send((peer, true, encode_msg_type(&MsgType::Server(ServerRes::UserCreated)))).unwrap();
                                },
                                Err(_) => {
                                    tx.send((peer, true, encode_msg_type(&MsgType::Server(ServerRes::Error("User already exist!.".to_string()))))).unwrap();
                                },
                            }
                        }
                        _ => {}
                    }
                },
                msg = rx.recv() => {
                    let (sender_addr, to_me, msg) = msg.unwrap();
                    if to_me {
                        if sender_addr == addr {
                            writer.write_all(&msg[..]).await.unwrap();
                        }
                        continue;
                    }
                    if sender_addr != addr {
                        writer.write_all(&msg[..]).await.unwrap();
                    }
                    // println!("{:?}", msg);
                }
            }
        }
    });
}
