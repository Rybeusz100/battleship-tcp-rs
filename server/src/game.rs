use crate::session::{self, Player};
use async_std::task;
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::mpsc};
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
    pub turn: Uuid,
    pub player_1: Player,
    pub player_2: Player,
}

impl Game {
    pub fn new(player_1: Player, player_2: Player) -> Self {
        Self {
            id: Uuid::new_v4(),
            turn: player_1.id,
            player_1,
            player_2,
        }
    }
}

pub struct Message {
    pub player_id: Uuid,
    pub content: MessageContent,
}

pub enum MessageContent {
    Disconnect,
    Shoot((u8, u8)),
}

pub fn start_game(mut game: Game) {
    println!("Starting game");

    let (tx, rx) = mpsc::channel();

    task::spawn(async move {
        let tx1 = tx.clone();
        let tx2 = tx.clone();
        game.player_1
            .tx
            .send(session::Message::StartGame(tx1))
            .await
            .ok();
        game.player_2
            .tx
            .send(session::Message::StartGame(tx2))
            .await
            .ok();
        while let Ok(msg) = rx.recv() {
            let (current_player, mut other_player) = if msg.player_id == game.player_1.id {
                (&game.player_1, &mut game.player_2)
            } else {
                (&game.player_2, &mut game.player_1)
            };

            match msg.content {
                MessageContent::Disconnect => {
                    // inform other player, end game
                    other_player
                        .tx
                        .send_blocking(session::Message::Disconnect)
                        .ok();
                    break;
                }
                MessageContent::Shoot(coords) => {
                    if game.turn != current_player.id {
                        continue;
                    }

                    handle_shot(coords, &mut other_player);
                }
            }
        }
    });
}

fn handle_shot(coords: (u8, u8), enemy: &mut Player) {
    if coords.0 > 9 || coords.1 > 9 {
        return;
    }

    // TODO check shot and modify enemy accordingly
}
