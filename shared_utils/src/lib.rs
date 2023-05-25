use serde::{Deserialize, Serialize};

use tokio::{io::AsyncReadExt, net::tcp::ReadHalf};

pub const MSG_SIZE_BYTES: usize = std::mem::size_of::<u32>();

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MsgDataType {
    Text(String),
    // Image
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Msg {
    pub username: String,
    pub data: MsgDataType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginMsg {
    pub username: String,
    pub password: String
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MsgType {
    Msg(Msg),
    Login(LoginMsg),
    Register(LoginMsg),
}

pub fn encode_str(str: &String) -> Vec<u8> {
    let mut buf = Vec::new();

    let mut offset: u8 = 0;
    let msg_size = str.len();
    buf.reserve(MSG_SIZE_BYTES + msg_size);

    // writing the msg size
    for _ in 0..MSG_SIZE_BYTES {
        buf.push(((msg_size >> offset) & 0xFF) as u8);
        offset += 8;
    }

    buf.splice(MSG_SIZE_BYTES.., str.bytes());

    buf
}

pub fn encode_bytes(bytes: Vec<u8>) -> Vec<u8> {
    let mut buf = Vec::new();

    let mut offset: u8 = 0;
    let msg_size = bytes.len();
    buf.reserve(MSG_SIZE_BYTES + msg_size);

    // writing the msg size
    for _ in 0..MSG_SIZE_BYTES {
        buf.push(((msg_size >> offset) & 0xFF) as u8);
        offset += 8;
    }

    buf.splice(MSG_SIZE_BYTES.., bytes);

    buf
}

pub fn encode_msg(msg: &MsgType) -> Vec<u8> {
    let mut buf = Vec::new();
    let serialized = serde_json::to_string(msg).unwrap();

    let mut offset: u8 = 0;
    let msg_size = serialized.len();
    buf.reserve(MSG_SIZE_BYTES + msg_size);

    // writing the msg size
    for _ in 0..MSG_SIZE_BYTES {
        buf.push(((msg_size >> offset) & 0xFF) as u8);
        offset += 8;
    }

    buf.splice(MSG_SIZE_BYTES.., serialized.bytes());

    buf
}

pub fn decode_header(data: &[u8]) -> u32 {
    let mut offset = 0;
    let mut value: u32 = 0;

    for i in 0..MSG_SIZE_BYTES {
        value |= u32::from(data[i]) << offset;
        offset += 8;
    }

    return value;
}

pub fn decode_msg_type(data: &Vec<u8>) -> Result<MsgType, serde_json::Error> {
    serde_json::from_str::<MsgType>(std::str::from_utf8(data).unwrap())
}

pub fn decode_msg(data: &Vec<u8>) -> Option<Msg> {
    if let Ok(mgs_type) = decode_msg_type(data) {
        match mgs_type {
            MsgType::Msg(msg) => return Some(msg),
            _ => return None
        };
    }
    None
}

pub async fn read_from_socket(reader: &mut ReadHalf<'_>, mut msg_len_buf: &mut Vec<u8>) -> Vec<u8> {
    reader.read(&mut msg_len_buf).await.unwrap();
    let msg_len = decode_header(&msg_len_buf[..]);
    let mut buf = vec![0; msg_len as usize];
    reader.read(&mut buf).await.unwrap();

    buf
}
