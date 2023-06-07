use serde::{Deserialize, Serialize};

pub type ClientBoard = [[bool; 10]; 10];

#[derive(Serialize, Deserialize)]
pub enum ClientToServer {
    SetBoard(ClientBoard),
}
