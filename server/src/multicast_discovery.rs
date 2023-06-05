use async_std::net::UdpSocket;
use async_std::task;
use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

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
            if let Ok(peer) = receive_multicast(&socket).await {
                println!("Received discovery message from {:?}", peer);
                let _ = socket.send_to(response_addr.as_bytes(), peer).await;
            }
        }
    });
}

pub async fn receive_multicast(socket: &UdpSocket) -> io::Result<SocketAddr> {
    let mut buf = [0; 100];
    let (_message_size, peer) = socket.recv_from(&mut buf).await?;
    Ok(peer)
}
