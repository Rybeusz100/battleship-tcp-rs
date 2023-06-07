use async_std::{
    io::{ReadExt, WriteExt},
    net::TcpStream,
};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

pub type ClientBoard = [[bool; 10]; 10];

#[derive(Serialize, Deserialize)]
pub enum ClientToServer {
    SetBoard(ClientBoard),
}

pub async fn send_message<T: Serialize>(stream: &mut TcpStream, msg: T) -> anyhow::Result<()> {
    let data: Vec<u8> = bincode::serialize(&msg)?;
    let data_len = data.len() as u32;

    let mut len_msg = vec![];
    len_msg.write_u32::<BigEndian>(data_len)?;
    stream.write_all(&len_msg).await?;

    stream.write_all(&data).await?;

    Ok(())
}

pub async fn receive_message<T: for<'a> Deserialize<'a>>(
    stream: &mut TcpStream,
) -> anyhow::Result<T> {
    let mut length_buf = [0; 4];
    let mut msg_buf = [0; 2048];

    stream.read_exact(&mut length_buf).await?;
    let message_length = Cursor::new(&length_buf).read_u32::<BigEndian>()? as usize;

    stream.read_exact(&mut msg_buf[..message_length]).await?;

    let msg: T = bincode::deserialize(&msg_buf)?;
    Ok(msg)
}
