use async_std::{net::TcpStream, task};
use shared::{send_message, ClientToServer};
use std::{
    io,
    net::{Ipv4Addr, UdpSocket},
    thread,
    time::Duration,
};

async fn run_client() -> io::Result<()> {
    let multicast_addr = "239.255.255.250:1901";
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let mut buf = [0; 100];

    socket
        .join_multicast_v4(&Ipv4Addr::new(239, 255, 255, 250), &Ipv4Addr::UNSPECIFIED)
        .unwrap();

    socket
        .send_to("Discovery message".as_bytes(), multicast_addr)
        .unwrap();

    let response_size = socket.recv(&mut buf).unwrap();
    let server_addr = std::str::from_utf8(&buf[..response_size]).unwrap();
    println!("Server addr: {}", server_addr);

    let mut stream = TcpStream::connect(server_addr).await.unwrap();

    let msg = ClientToServer::SetBoard([[false; 10]; 10]);
    send_message(&mut stream, msg).await.unwrap();

    thread::sleep(Duration::from_secs(5));

    let msg = ClientToServer::SetBoard([[true; 10]; 10]);
    send_message(&mut stream, msg).await.unwrap();

    Ok(())
}

fn main() -> io::Result<()> {
    task::block_on(async {
        run_client().await?;
        Ok(())
    })
}
