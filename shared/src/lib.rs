use async_std::{
    io::{ReadExt, WriteExt},
    net::TcpStream,
};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

pub type ClientBoard = [[bool; 10]; 10];
pub type AllyBoard = [[AllyField; 10]; 10];
pub type EnemyBoard = [[EnemyField; 10]; 10];

#[derive(Serialize, Deserialize)]
pub enum ClientToServer {
    SetBoard(ClientBoard),
    Shoot((u8, u8)),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerToClient {
    UpdateAlly(AllyBoard),
    UpdateEnemy(EnemyBoard),
    Disconnect(DisconnectReason),
    YourTurn,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DisconnectReason {
    Win,
    Defeat,
    Error,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum AllyField {
    Free,
    Occupied,
    Missed,
    Hit,
    Sank,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum EnemyField {
    Unknown,
    Hit,
    Sank,
    Missed,
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

impl From<AllyField> for char {
    fn from(field: AllyField) -> char {
        match field {
            AllyField::Free => '🟦',
            AllyField::Occupied => '⚪',
            AllyField::Missed => '🟫',
            AllyField::Hit => '🟡',
            AllyField::Sank => '🔴',
        }
    }
}

impl From<EnemyField> for char {
    fn from(field: EnemyField) -> char {
        match field {
            EnemyField::Unknown => '🟦',
            EnemyField::Hit => '🟡',
            EnemyField::Sank => '🔴',
            EnemyField::Missed => '🟫',
        }
    }
}
