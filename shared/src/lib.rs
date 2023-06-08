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
    Shoot((u8, u8)),
}

pub async fn send_message<T: Serialize>(stream: &mut TcpStream, msg: T) -> anyhow::Result<()> {
    let data: Vec<u8> = bincode::serialize(&msg)?;
    let data_len = data.len() as u32;

    let mut len_msg = Vec::with_capacity(4);
    len_msg.write_u32::<BigEndian>(data_len)?;
    stream.write_all(&len_msg).await?;

    stream.write_all(&data).await?;

    Ok(())
}

pub async fn receive_message<T: for<'a> Deserialize<'a>>(
    stream: &mut TcpStream,
) -> anyhow::Result<T> {
    let mut buf = [0; 2048];

    stream.read_exact(&mut buf[..4]).await?;
    let message_length = Cursor::new(&buf[..4]).read_u32::<BigEndian>()? as usize;

    stream.read_exact(&mut buf[..message_length]).await?;

    let msg: T = bincode::deserialize(&buf)?;
    Ok(msg)
}
