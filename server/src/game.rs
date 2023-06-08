use crate::session::{self, Player};
use async_std::task;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::mpsc::{self, Sender},
};
use uuid::Uuid;

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
    Disconnect,
}

pub fn start_game(game: Game) -> Sender<Message> {
    let (tx, rx) = mpsc::channel();

    task::spawn(async move {
        game.player_1
            .tx
            .send(session::Message::StartGame)
            .await
            .ok();
        game.player_2
            .tx
            .send(session::Message::StartGame)
            .await
            .ok();
        while let Ok(msg) = rx.recv() {}
    });

    tx
}
