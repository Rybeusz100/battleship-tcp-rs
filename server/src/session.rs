use crate::{
    game::{Board, FieldState, Player, INITIAL_SHIPS_COUNT},
    manager, CONNECTIONS_COUNT,
};
use async_std::net::TcpStream;
use async_std::task;
use shared::{receive_message, ClientBoard, ClientToServer};
use std::{
    sync::{atomic::Ordering, mpsc::Sender},
};
use uuid::Uuid;

enum ClientState {
    Connected,
    Ready,
    Playing,
}

pub fn handle_client(mut stream: TcpStream, manager_tx: Sender<manager::Message>) {
    task::spawn(async move {
        println!("Client connected");
        CONNECTIONS_COUNT.fetch_add(1, Ordering::SeqCst);
        let state = ClientState::Connected;
        let player_id = Uuid::new_v4();

        // TODO timeout
        while let Ok(msg) = receive_message(&mut stream).await {
            match state {
                ClientState::Connected => {
                    if let ClientToServer::SetBoard(client_board) = msg {
                        println!("Received board {:?}", client_board);
                        if verify_board(&client_board) {
                            let board = create_board(client_board);
                            let player = Player {
                                id: player_id,
                                board,
                                alive_ships: INITIAL_SHIPS_COUNT,
                            };
                            manager_tx.send(manager::Message::Ready(player)).unwrap();
                        }
                    }
                }
                _ => (),
            }
        }
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
