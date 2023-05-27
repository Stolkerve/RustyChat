use postcard::{from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
extern crate alloc;
use alloc::vec::Vec;

pub const MSG_SIZE_BYTES: usize = std::mem::size_of::<u32>();

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MsgDataType {
    Text(String),
    Image(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserMsg {
    pub username: String,
    pub data: MsgDataType,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerMsg {
    pub username: String,
    pub data: MsgDataType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginMsg {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenMsg {
    pub token: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerRes {
    Error(String),
    UserToken(TokenMsg),
    UserCreated,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MsgType {
    MsgIn(ServerMsg),
    MsgOut(UserMsg),
    Login(LoginMsg),
    Signup(LoginMsg),
    Server(ServerRes),
}

// Write the msg header and body
fn write_header_to_buf(bytes: Vec<u8>, buf: &mut Vec<u8>) {
    let mut offset: u8 = 0;
    let msg_size = bytes.len();
    buf.reserve(MSG_SIZE_BYTES + msg_size);

    // writing the msg size
    for _ in 0..MSG_SIZE_BYTES {
        buf.push(((msg_size >> offset) & 0xFF) as u8);
        offset += 8;
    }

    buf.splice(MSG_SIZE_BYTES.., bytes);
}

pub fn encode_bytes(bytes: Vec<u8>) -> Vec<u8> {
    let mut buf = Vec::new();
    write_header_to_buf(bytes, &mut buf);
    buf
}

pub fn encode_msg_type(msg: &MsgType) -> Vec<u8> {
    let mut buf = Vec::new();
    let serialized: Vec<u8> = to_allocvec(msg).unwrap();

    write_header_to_buf(serialized, &mut buf);

    buf
}

pub fn decode_header(data: &[u8]) -> u32 {
    let mut offset = 0;
    let mut value: u32 = 0;

    for byte in data.iter().take(MSG_SIZE_BYTES) {
        value |= u32::from(*byte) << offset;
        offset += 8;
    }

    value
}

pub fn decode_msg_type(data: &[u8]) -> Result<MsgType, postcard::Error> {
    from_bytes(data)
}