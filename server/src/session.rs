use crate::{
    game::{Board, FieldState, INITIAL_SHIPS_COUNT},
    manager, CONNECTIONS_COUNT,
};
use async_std::task;
use async_std::{
    channel::{self, Sender as AsyncSender},
    net::TcpStream,
};
use async_stream::stream;
use futures::{future::Either, pin_mut, stream::select, StreamExt};
use shared::{receive_message, ClientBoard, ClientToServer};
use std::sync::{atomic::Ordering, mpsc::Sender};
use uuid::Uuid;

enum ClientState {
    Connected,
    Ready,
    Playing,
}

pub enum Message {
    StartGame,
    Disconnect,
}

pub struct Player {
    pub id: Uuid,
    pub tx: AsyncSender<Message>,
    pub board: Board,
    pub alive_ships: u8,
}

pub fn handle_client(mut stream: TcpStream, manager_tx: Sender<manager::Message>) {
    task::spawn(async move {
        println!("Client connected");
        CONNECTIONS_COUNT.fetch_add(1, Ordering::SeqCst);
        let mut state = ClientState::Connected;
        let player_id = Uuid::new_v4();
        let (tx, rx) = channel::unbounded();

        let combined = select(
            stream! { loop { yield Either::Left(rx.recv().await) } },
            stream! { loop { yield Either::Right(receive_message::<ClientToServer>(&mut stream).await) } },
        );

        pin_mut!(combined);

        // TODO timeout
        while let Some(msg) = combined.next().await {
            match msg {
                Either::Left(Ok(msg)) => match msg {
                    Message::Disconnect => {
                        break;
                    }
                    Message::StartGame => {
                        println!("Starting game");
                    }
                },
                Either::Right(Ok(msg)) => match state {
                    ClientState::Connected => {
                        if let ClientToServer::SetBoard(client_board) = msg {
                            println!("Received board {:?}", client_board);
                            if verify_board(&client_board) {
                                let board = create_board(client_board);
                                let player = Player {
                                    id: player_id,
                                    tx: tx.clone(),
                                    board,
                                    alive_ships: INITIAL_SHIPS_COUNT,
                                };
                                manager_tx.send(manager::Message::Ready(player)).unwrap();
                                state = ClientState::Ready;
                            }
                        }
                    }
                    _ => (),
                },
                _ => {
                    println!("select! error");
                    break;
                }
            }
        }
        manager_tx
            .send(manager::Message::Disconnect(player_id))
            .unwrap();
        println!("Client disconnected");
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

/// verifies if all ships are used and if there's empty space between ships
fn verify_board(board: &ClientBoard) -> bool {
    // TODO verify lengths
    // TODO verify spacing
    true
}
