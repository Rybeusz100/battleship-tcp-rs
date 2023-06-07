use crate::{
    game::{Board, FieldState, INITIAL_SHIPS_COUNT},
    manager, CONNECTIONS_COUNT,
};
use async_std::net::TcpStream;
use async_std::task;
use shared::{receive_message, ClientBoard, ClientToServer};
use std::sync::{atomic::Ordering, mpsc::{Sender, self}};
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
    pub tx: Sender<Message>,
    pub board: Board,
    pub alive_ships: u8,
}

pub fn handle_client(mut stream: TcpStream, manager_tx: Sender<manager::Message>) {
    task::spawn(async move {
        println!("Client connected");
        CONNECTIONS_COUNT.fetch_add(1, Ordering::SeqCst);
        let mut state = ClientState::Connected;
        let player_id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel();

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
            }
        }
        manager_tx.send(manager::Message::Disconnect(player_id)).unwrap();
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
