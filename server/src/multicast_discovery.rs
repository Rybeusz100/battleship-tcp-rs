use async_std::net::UdpSocket;
use async_std::task;
use std::net::{Ipv4Addr, SocketAddrV4};

pub fn start_multicast_discovery(multicast_addr: SocketAddrV4, response_addr: String) {
    task::spawn(async move {
        let socket = UdpSocket::bind(SocketAddrV4::new(
            Ipv4Addr::new(0, 0, 0, 0),
            multicast_addr.port(),
        ))
        .await
        .unwrap();

        socket
            .join_multicast_v4(*multicast_addr.ip(), Ipv4Addr::UNSPECIFIED)
            .unwrap();

        loop {
            let msg = receive_multicast(&socket).await;
            if let Ok(msg) = msg {
                println!("Received discovery message");
                let _ = socket
                    .send_to(
                        response_addr.as_bytes(),
                        multicast_addr.ip().to_string() + ":" + &msg,
                    )
                    .await;
            }
        }
    });
}

pub async fn receive_multicast(socket: &UdpSocket) -> anyhow::Result<String> {
    let mut buf = [0; 100];
    let message_size = socket.recv(&mut buf).await?;
    let message = std::str::from_utf8(&buf[..message_size])?.to_owned();
    Ok(message)
}
