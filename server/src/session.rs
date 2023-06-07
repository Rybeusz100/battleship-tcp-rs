use crate::game::{Board, FieldState};
use async_std::io::ReadExt;
use async_std::net::TcpStream;
use async_std::task;
use byteorder::{BigEndian, ReadBytesExt};
use shared::{ClientBoard, ClientToServer};
use std::io::Cursor;

enum ClientState {
    Connected,
    Ready,
    Playing,
}

pub fn handle_client(mut stream: TcpStream) {
    task::spawn(async move {
        println!("Client connected");
        let state = ClientState::Connected;

        let mut length_buf = [0; 4];
        let mut msg_buf = [0; 2048];
        // TODO timeout
        while let Ok(message_size) = stream.read_exact(&mut length_buf).await {
            let message_length = Cursor::new(&length_buf).read_u32::<BigEndian>().unwrap() as usize;

            if let Ok(_) = stream.read_exact(&mut msg_buf[..message_length]).await {
                let msg: ClientToServer = match bincode::deserialize(&msg_buf) {
                    Ok(msg) => msg,
                    Err(why) => {
                        println!("Msg deserialization error: {}", why);
                        break;
                    }
                };

                match state {
                    ClientState::Connected => {
                        if let ClientToServer::SetBoard(client_board) = msg {
                            println!("Received board {:?}", client_board);
                            if verify_board(&client_board) {
                                let board = create_board(client_board);
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
        println!("Client disconnected");
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
