use crate::utils::{convert_to_bool_array, draw_board, parse_board_coordinates};
use async_std::{net::TcpStream, task};
use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use rand::seq::SliceRandom;
use shared::{receive_message, send_message, ClientToServer, ServerToClient};
use std::{
    env,
    io::{self, stdout, BufRead},
    net::{Ipv4Addr, UdpSocket},
    time::Duration,
};
use {shared::EnemyField, std::thread};

mod utils;

async fn run_client(automatic: bool) -> io::Result<()> {
    stdout().execute(Clear(ClearType::All))?;
    stdout().execute(MoveTo(0, 0))?;

    let mut rng = rand::thread_rng();
    let mut enemy_board = [[EnemyField::Unknown; 10]; 10];

    // TODO read input
    let board = [
        [1, 0, 1, 0, 1, 1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
        [0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        [0, 0, 0, 1, 1, 1, 1, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let multicast_addr = "239.255.255.250:1901";
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let mut buf = [0; 100];

    socket
        .join_multicast_v4(&Ipv4Addr::new(239, 255, 255, 250), &Ipv4Addr::UNSPECIFIED)
        .unwrap();

    println!("Waiting for server...");
    let server_addr;
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    loop {
        socket
            .send_to("Discovery message".as_bytes(), multicast_addr)
            .unwrap();

        if let Ok(response_size) = socket.recv(&mut buf) {
            server_addr = std::str::from_utf8(&buf[..response_size]).unwrap();
            println!("Found server: {}", server_addr);
            break;
        }
    }

    let mut stream = TcpStream::connect(server_addr).await.unwrap();

    let msg = ClientToServer::SetBoard(convert_to_bool_array(board));
    send_message(&mut stream, msg).await.unwrap();

    while let Ok(msg) = receive_message::<ServerToClient>(&mut stream).await {
        match msg {
            ServerToClient::UpdateAlly(board) => {
                draw_board(board, (0, 4))?;
            }
            ServerToClient::UpdateEnemy(board) => {
                if automatic {
                    enemy_board = board;
                }
                draw_board(board, (30, 4))?;
            }
            ServerToClient::Disconnect(reason) => {
                stdout().execute(MoveTo(0, 16))?;
                println!("Disconnected because: {:?}", reason);
                break;
            }
            ServerToClient::YourTurn => {
                stdout().execute(MoveTo(0, 16))?;
                println!("Your turn!");

                if automatic {
                    let unknown_fields: Vec<(u8, u8)> = enemy_board
                        .iter()
                        .enumerate()
                        .flat_map(|(i, row)| {
                            row.iter().enumerate().filter_map(move |(j, field)| {
                                if *field == EnemyField::Unknown {
                                    Some((i as u8, j as u8))
                                } else {
                                    None
                                }
                            })
                        })
                        .collect();
                    let coords = unknown_fields.choose(&mut rng).copied().unwrap();
                    thread::sleep(Duration::from_millis(100));
                    send_message(&mut stream, ClientToServer::Shoot(coords))
                        .await
                        .ok();
                    continue;
                }

                loop {
                    let mut input = String::new();
                    io::stdin().lock().read_line(&mut input).unwrap();

                    input = input.to_uppercase().trim().to_string();

                    match parse_board_coordinates(&input) {
                        Ok(coords) => {
                            send_message(&mut stream, ClientToServer::Shoot(coords))
                                .await
                                .ok();
                            stdout()
                                .execute(MoveTo(0, 16))?
                                .execute(Clear(ClearType::FromCursorDown))?;
                            break;
                        }
                        Err(why) => {
                            println!("{}", why);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    task::block_on(async {
        run_client(args.len() > 1 && args[1] == "automatic").await?;
        Ok(())
    })
}
