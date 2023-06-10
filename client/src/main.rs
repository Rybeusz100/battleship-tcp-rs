use crate::utils::{convert_to_bool_array, parse_board_coordinates};
use async_std::{net::TcpStream, task};
#[cfg(feature = "automatic")]
use rand::Rng;
use shared::{
    receive_message, send_message, AllyBoard, AllyField, ClientToServer, EnemyBoard, EnemyField,
    ServerToClient,
};
use std::{
    io::{self, BufRead},
    net::{Ipv4Addr, UdpSocket},
    time::Duration,
};

mod utils;

async fn run_client() -> io::Result<()> {
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
    let mut ally_board: AllyBoard = [[AllyField::Free; 10]; 10];
    let mut enemy_board: EnemyBoard = [[EnemyField::Unknown; 10]; 10];

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
                ally_board = board;
            }
            ServerToClient::UpdateEnemy(board) => enemy_board = board,
            ServerToClient::Disconnect(reason) => {
                println!("Disconnected because: {:?}", reason);
                break;
            }
            ServerToClient::YourTurn => {
                println!("Your turn!");
                #[cfg(feature = "automatic")]
                {
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
