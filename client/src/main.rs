use std::net::{Ipv4Addr, UdpSocket};

fn main() {
    let multicast_addr = "239.255.255.250:1901";
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let mut buf = [0; 100];

    socket
        .join_multicast_v4(&Ipv4Addr::new(239, 255, 255, 250), &Ipv4Addr::UNSPECIFIED)
        .unwrap();

    socket
        .send_to(
            socket.local_addr().unwrap().port().to_string().as_bytes(),
            multicast_addr,
        )
        .unwrap();

    let (response_size, _) = socket.recv_from(&mut buf).unwrap();
    let response = std::str::from_utf8(&buf[..response_size]).unwrap();
    println!("Server addr: {}", response);
}
