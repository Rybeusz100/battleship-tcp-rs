use async_std::{net::TcpStream, task};
use shared::{
    receive_message, send_message, AllyBoard, AllyField, ClientToServer, EnemyBoard, EnemyField,
    ServerToClient,
};
use std::{
    io::{self, BufRead},
    net::{Ipv4Addr, UdpSocket},
};

async fn run_client() -> io::Result<()> {
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

    socket
        .send_to("Discovery message".as_bytes(), multicast_addr)
        .unwrap();

    println!("Waiting for server...");
    let response_size = socket.recv(&mut buf).unwrap();
    let server_addr = std::str::from_utf8(&buf[..response_size]).unwrap();
    println!("Found server: {}", server_addr);

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
                    send_message(&mut stream, ClientToServer::Shoot((0, 0)))
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

fn convert_to_bool_array(input: [[i32; 10]; 10]) -> [[bool; 10]; 10] {
    let mut bool_array = [[false; 10]; 10];

    for i in 0..10 {
        for j in 0..10 {
            bool_array[i][j] = input[i][j] != 0;
        }
    }

    bool_array
}

fn parse_board_coordinates(input: &str) -> Result<(u8, u8), String> {
    if input.len() != 2 {
        return Err("Invalid input length".to_string());
    }

    let x_char = input.chars().next().ok_or("Invalid input")?;
    let y_char = input.chars().nth(1).ok_or("Invalid input")?;

    let x_offset = b'A';
    let y_offset = b'0';

    let x_val = x_char as u8;
    let y_val = y_char as u8;

    if x_val < b'A' || y_val < b'0' {
        return Err("Invalid board coordinates".to_string());
    }

    let x = x_val - x_offset;
    let y = y_val - y_offset;

    if x > 9 || y > 9 {
        return Err("Invalid board coordinates".to_string());
    }

    Ok((x, y))
}
