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
