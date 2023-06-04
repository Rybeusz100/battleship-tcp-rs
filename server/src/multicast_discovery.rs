use async_std::net::UdpSocket;
use std::{
    io,
    net::{Ipv4Addr, SocketAddrV4},
};

pub async fn create_multicast_socket(multicast_addr: &SocketAddrV4) -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        multicast_addr.port(),
    ))
    .await?;
    socket.join_multicast_v4(*multicast_addr.ip(), Ipv4Addr::UNSPECIFIED)?;
    Ok(socket)
}

pub async fn receive_multicast(socket: &UdpSocket) -> anyhow::Result<String> {
    let mut buf = [0; 100];
    let message_size = socket.recv(&mut buf).await?;
    let message = std::str::from_utf8(&buf[..message_size])?.to_owned();
    Ok(message)
}
