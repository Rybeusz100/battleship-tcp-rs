use lazy_static::lazy_static;
use std::{collections::HashMap, sync::mpsc::{self, Sender}};
use uuid::Uuid;
use async_std::task;
use crate::session::Player;

pub const INITIAL_SHIPS_COUNT: u8 = 7;

lazy_static! {
    static ref REQUIRED_SHIPS: HashMap<u8, u8> = {
        let mut m = HashMap::new();
        m.insert(1, 2);
        m.insert(2, 2);
        m.insert(3, 1);
        m.insert(4, 1);
        m.insert(5, 1);
        m
    };
}

pub type Board = [[FieldState; 10]; 10];

#[derive(Clone, Copy)]
pub enum FieldState {
    Free,
    Occupied,
    Missed,
    Hit,
    Sank,
}

pub struct Game {
    pub id: Uuid,
    pub player_1: Player,
    pub player_2: Player,
}

pub enum Message {
    Disconnect
}

pub fn start_game(game: Game) -> Sender<Message> {
    let (tx, rx) = mpsc::channel();

    task::spawn(async move {
        while let Ok(msg) = rx.recv() {

        }
    });

    tx
}
