use crate::session::{self, Player};
use async_std::task;
use shared::{AllyBoard, AllyField, EnemyBoard, EnemyField};
use std::sync::mpsc;
use uuid::Uuid;

pub const INITIAL_SHIPS_COUNT: u8 = 7;

pub const REQUIRED_SHIPS: [u8; 11] = [0, 2, 2, 1, 1, 1, 0, 0, 0, 0, 0];

pub type Board = [[FieldState; 10]; 10];
pub type FieldState = AllyField;

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
            let (current_player, other_player) = if msg.player_id == game.player_1.id {
                (&game.player_1, &mut game.player_2)
            } else {
                (&game.player_2, &mut game.player_1)
            };

            match msg.content {
                MessageContent::Disconnect => {
                    // inform other player, end game
                    other_player
                        .tx
                        .send_blocking(session::Message::Disconnect(
                            shared::DisconnectReason::Error,
                        ))
                        .ok();
                    break;
                }
                MessageContent::Shoot(coords) => {
                    if game.turn != current_player.id {
                        continue;
                    }

                    handle_shot(coords, other_player);

                    let (ally_board, enemy_board) = prepare_boards_to_send(&other_player.board);
                    current_player
                        .tx
                        .send_blocking(session::Message::UpdateEnemy(enemy_board))
                        .ok();
                    other_player
                        .tx
                        .send_blocking(session::Message::UpdateAlly(ally_board))
                        .ok();

                    if other_player.alive_ships == 0 {
                        current_player
                            .tx
                            .send_blocking(session::Message::Disconnect(
                                shared::DisconnectReason::Win,
                            ))
                            .ok();
                        other_player
                            .tx
                            .send_blocking(session::Message::Disconnect(
                                shared::DisconnectReason::Defeat,
                            ))
                            .ok();
                        break;
                    }

                    game.turn = other_player.id;
                }
            }
        }
    });
}

fn handle_shot(coords: (u8, u8), enemy: &mut Player) {
    if coords.0 > 9 || coords.1 > 9 {
        return;
    }

    let mut board = enemy.board;
    let (y, x) = (coords.0 as usize, coords.1 as usize);
    let field = &mut board[y][x];

    if *field == FieldState::Occupied {
        *field = FieldState::Hit;

        // check if the ship sank
        let mut sank = true;
        let mut ship_fields = vec![(y, x)];

        let mut horizontal = false;
        if x > 0 {
            if let Some(FieldState::Hit | FieldState::Occupied) = board[y].get(x - 1) {
                horizontal = true;
            }
        }
        if let Some(FieldState::Hit | FieldState::Occupied) = board[y].get(x + 1) {
            horizontal = true;
        }

        if horizontal {
            let mut x_2 = x;
            while x_2 > 0 {
                x_2 -= 1;
                match board[y].get(x_2) {
                    Some(FieldState::Hit) => {
                        ship_fields.push((y, x_2));
                    }
                    Some(FieldState::Occupied) => {
                        sank = false;
                    }
                    _ => (),
                }
            }
            x_2 = x + 1;
            while let Some(field) = board[y].get(x_2) {
                match field {
                    FieldState::Hit => {
                        ship_fields.push((y, x_2));
                    }
                    FieldState::Occupied => sank = false,
                    _ => (),
                }
                x_2 += 1;
            }
        } else {
            let mut y_2 = y;
            while y_2 > 0 {
                y_2 -= 1;
                match board[y_2][x] {
                    FieldState::Hit => {
                        ship_fields.push((y_2, x));
                    }
                    FieldState::Occupied => {
                        sank = false;
                    }
                    _ => (),
                }
            }
            y_2 = y + 1;
            while y_2 < 10 {
                match board[y_2][x] {
                    FieldState::Hit => {
                        ship_fields.push((y_2, x));
                    }
                    FieldState::Occupied => sank = false,
                    _ => (),
                }
                y_2 += 1;
            }
        }

        if sank {
            enemy.alive_ships -= 1;
            for field in ship_fields {
                board[field.0][field.1] = FieldState::Sank;
            }
        }
    } else if *field == FieldState::Free {
        *field = FieldState::Missed;
    }
}

fn prepare_boards_to_send(board: &Board) -> (AllyBoard, EnemyBoard) {
    let mut e_board = [[EnemyField::Unknown; 10]; 10];

    for (y, row) in board.iter().enumerate() {
        for (x, field) in row.iter().enumerate() {
            e_board[y][x] = match field {
                AllyField::Free | AllyField::Occupied => EnemyField::Unknown,
                AllyField::Hit => EnemyField::Hit,
                AllyField::Missed => EnemyField::Missed,
                AllyField::Sank => EnemyField::Sank,
            };
        }
    }

    (*board, e_board)
}
