use crate::{
    game::{self, Board, FieldState, INITIAL_SHIPS_COUNT, REQUIRED_SHIPS},
    manager, CONNECTIONS_COUNT,
};
use async_std::task;
use async_std::{
    channel::{self, Sender},
    net::TcpStream,
};
use async_stream::stream;
use futures::{future::Either, pin_mut, stream::select, StreamExt};
use log::{debug, info};
use shared::{
    receive_message, send_message, AllyBoard, ClientBoard, ClientToServer, DisconnectReason,
    EnemyBoard, ServerToClient,
};
use std::{net::SocketAddr, sync::atomic::Ordering};
use uuid::Uuid;

enum ClientState {
    Connected,
    Ready,
    Playing,
}

pub enum Message {
    StartGame(Sender<game::Message>),
    UpdateAlly(AllyBoard),
    UpdateEnemy(EnemyBoard),
    Disconnect(DisconnectReason),
    YourTurn,
}

pub struct Player {
    pub address: SocketAddr,
    pub id: Uuid,
    pub tx: Sender<Message>,
    pub board: Board,
    pub alive_ships: u8,
}

pub fn handle_client(mut stream: TcpStream, manager_tx: Sender<manager::Message>) {
    let peer_addr = if let Ok(addr) = stream.peer_addr() {
        addr
    } else {
        return;
    };

    task::spawn(async move {
        debug!("{} connected", peer_addr);
        CONNECTIONS_COUNT.fetch_add(1, Ordering::SeqCst);
        let mut state = ClientState::Connected;
        let player_id = Uuid::new_v4();
        let (tx, rx) = channel::unbounded();
        let mut game_tx = None;
        let mut send_stream = stream.clone();

        let combined = select(
            stream! { loop { yield Either::Left(rx.recv().await) } },
            stream! { loop { yield Either::Right(receive_message::<ClientToServer>(&mut stream).await) } },
        );

        pin_mut!(combined);

        while let Some(msg) = combined.next().await {
            match msg {
                Either::Left(Ok(msg)) => match msg {
                    Message::Disconnect(reason) => {
                        send_message(&mut send_stream, ServerToClient::Disconnect(reason))
                            .await
                            .ok();
                        break;
                    }
                    Message::StartGame(tx) => {
                        state = ClientState::Playing;
                        game_tx = Some(tx);
                    }
                    Message::UpdateAlly(board) => {
                        send_message(&mut send_stream, ServerToClient::UpdateAlly(board))
                            .await
                            .ok();
                    }
                    Message::UpdateEnemy(board) => {
                        send_message(&mut send_stream, ServerToClient::UpdateEnemy(board))
                            .await
                            .ok();
                    }
                    Message::YourTurn => {
                        send_message(&mut send_stream, ServerToClient::YourTurn)
                            .await
                            .ok();
                    }
                },
                Either::Right(Ok(msg)) => match state {
                    ClientState::Connected => {
                        if let ClientToServer::SetBoard(client_board) = msg {
                            if verify_board(&client_board) {
                                let board = create_board(client_board);
                                let player = Player {
                                    address: peer_addr,
                                    id: player_id,
                                    tx: tx.clone(),
                                    board,
                                    alive_ships: INITIAL_SHIPS_COUNT,
                                };
                                manager_tx
                                    .send(manager::Message::Ready(player))
                                    .await
                                    .unwrap();
                                state = ClientState::Ready;
                            } else {
                                send_message(
                                    &mut send_stream,
                                    ServerToClient::Disconnect(DisconnectReason::Error),
                                )
                                .await
                                .ok();
                                break;
                            }
                        }
                    }
                    ClientState::Playing => {
                        if let Some(game_tx) = game_tx.as_ref() {
                            if let ClientToServer::Shoot(coords) = msg {
                                game_tx
                                    .send(game::Message {
                                        player_id,
                                        content: game::MessageContent::Shoot(coords),
                                    })
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                    _ => (),
                },
                _ => {
                    break;
                }
            }
        }

        manager_tx
            .send(manager::Message::Disconnect(player_id))
            .await
            .unwrap();
        if let Some(tx) = game_tx {
            tx.send(game::Message {
                player_id,
                content: game::MessageContent::Disconnect,
            })
            .await
            .ok();
        }
        info!("{} disconnected", peer_addr);
        CONNECTIONS_COUNT.fetch_sub(1, Ordering::SeqCst);
    });
}

fn create_board(client_board: ClientBoard) -> Board {
    let mut board: Board = [[FieldState::Free; 10]; 10];
    for (y, row) in client_board.iter().enumerate() {
        for (x, field) in row.iter().enumerate() {
            if *field {
                board[y][x] = FieldState::Occupied;
            }
        }
    }
    board
}

/// Verifies if all ships are used and if there's empty space between ships
fn verify_board(board: &ClientBoard) -> bool {
    // verify lengths
    let mut checked = [[false; 10]; 10];

    let mut ship_lengths = [0; 11];
    for (y, row) in board.iter().enumerate() {
        for (x, field) in row.iter().enumerate() {
            if !field || checked[y][x] {
                continue;
            }

            let mut ship_length = 1;

            // check horizontal ship
            let mut x_2 = x + 1;
            while let Some(true) = board[y].get(x_2) {
                ship_length += 1;
                checked[y][x_2] = true;
                x_2 += 1;
            }

            if ship_length > 1 {
                ship_lengths[ship_length] += 1;
                continue;
            }

            // check vertical ship
            let mut y_2 = y + 1;
            while let Some(row) = board.get(y_2) {
                if row[x] {
                    ship_length += 1;
                    checked[y_2][x] = true;
                    y_2 += 1;
                } else {
                    break;
                }
            }

            ship_lengths[ship_length] += 1;
        }
    }

    if ship_lengths != REQUIRED_SHIPS {
        return false;
    }

    // verify spacing
    for (y, row) in board.iter().enumerate() {
        for (x, field) in row.iter().enumerate() {
            if !field {
                continue;
            }

            let mut neighbors = [false; 4];
            if let Some(true) = board[y].get(x + 1) {
                neighbors[0] = true;
            }
            if x > 0 {
                if let Some(true) = board[y].get(x - 1) {
                    neighbors[2] = true;
                }
            }
            if let Some(row) = board.get(y + 1) {
                if row[x] {
                    neighbors[1] = true;
                }
            }
            if y > 0 {
                if let Some(row) = board.get(y - 1) {
                    if row[x] {
                        neighbors[3] = true;
                    }
                }
            }

            if (neighbors[0] || neighbors[2]) && (neighbors[1] || neighbors[3]) {
                return false;
            }
        }
    }

    true
}
