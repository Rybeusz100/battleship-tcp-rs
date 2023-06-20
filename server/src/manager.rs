use crate::{
    game::{start_game, Game},
    session::Player,
};
use async_std::{
    channel::{self, Sender},
    task,
};
use log::debug;
use uuid::Uuid;

pub enum Message {
    Ready(Player),
    Disconnect(Uuid),
}

pub fn start_manager() -> Sender<Message> {
    let (tx, rx) = channel::unbounded();

    task::spawn(async move {
        let mut waiting_player: Option<Player> = None;

        while let Ok(msg) = rx.recv().await {
            match msg {
                Message::Ready(new_player) => {
                    debug!("{} ready", new_player.address);
                    if let Some(old_player) = waiting_player {
                        let new_game = Game::new(old_player, new_player);
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
