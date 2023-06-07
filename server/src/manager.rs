use crate::{game::{Game, start_game}, session::Player};
use async_std::task;
use std::{
    sync::mpsc::{self, Sender},
};
use uuid::Uuid;

pub enum Message {
    Ready(Player),
    Disconnect(Uuid)
}

pub fn start_manager() -> Sender<Message> {
    let (tx, rx) = mpsc::channel();

    task::spawn(async move {
        let mut waiting_player: Option<Player> = None;

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
                        waiting_player = None;
                        start_game(new_game);
                    } else {
                        waiting_player = Some(new_player);
                    }
                }
                Message::Disconnect(id) => {
                    if let Some(w_player) = waiting_player.as_ref() {
                        if w_player.id == id {
                            waiting_player = None;
                        }
                    }
                }
            }
        }
    });

    tx
}
