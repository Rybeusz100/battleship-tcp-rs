use crate::utils::{convert_to_bool_array, draw_board, parse_board_coordinates};
use async_std::{net::TcpStream, task};
use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use shared::{receive_message, send_message, ClientToServer, ServerToClient};
use std::{
    io::{self, stdout, BufRead},
    net::{Ipv4Addr, UdpSocket},
    time::Duration,
};
#[cfg(feature = "automatic")]
use {rand::Rng, std::thread};

mod utils;

async fn run_client() -> io::Result<()> {
    stdout().execute(Clear(ClearType::All))?;
    stdout().execute(MoveTo(0, 0))?;

    #[cfg(feature = "automatic")]
    let mut rng = rand::thread_rng();

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
                #[cfg(feature = "automatic")]
                {
                    thread::sleep(Duration::from_millis(100));
                    let coords = (rng.gen_range(0..=9), rng.gen_range(0..=9));
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
    task::block_on(async {
        run_client().await?;
        Ok(())
    })
}
