use crate::game::{Game, Player};
use async_std::task;
use std::{
    collections::HashMap,
    sync::mpsc::{self, Sender},
};
use uuid::Uuid;

pub enum Message {
    Ready(Player),
}

pub fn start_manager() -> Sender<Message> {
    let (tx, rx) = mpsc::channel::<Message>();

    task::spawn(async move {
        let mut waiting_player: Option<Player> = None;
        let mut games: HashMap<Uuid, Game> = HashMap::new();

        while let Ok(msg) = rx.recv() {
            match msg {
                Message::Ready(new_player) => {
                    println!("Player ready");
                    if let Some(old_player) = waiting_player {
                        let id = Uuid::new_v4();
                        let new_game = Game {
                            id,
                            player_1: old_player,
                            player_2: new_player,
                        };
                        games.insert(id, new_game);
                        waiting_player = None;
                    } else {
                        waiting_player = Some(new_player);
                    }
                }
            }
        }
    });

    tx
}
