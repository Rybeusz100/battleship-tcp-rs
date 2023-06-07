use byteorder::{BigEndian, WriteBytesExt};
use shared::ClientToServer;
use std::{
    io::Write,
    net::{Ipv4Addr, TcpStream, UdpSocket},
    thread,
    time::Duration,
};

fn send_message(stream: &mut TcpStream, msg: ClientToServer) {
    let data: Vec<u8> = bincode::serialize(&msg).unwrap();
    let data_len = data.len() as u32;
    stream.write_u32::<BigEndian>(data_len).unwrap();
    stream.write_all(&data).unwrap();
}

fn main() {
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

    let mut stream = TcpStream::connect(server_addr).unwrap();

    let msg = ClientToServer::SetBoard([[false; 10]; 10]);
    send_message(&mut stream, msg);

    thread::sleep(Duration::from_secs(5));

    let msg = ClientToServer::SetBoard([[true; 10]; 10]);
    send_message(&mut stream, msg);
}
