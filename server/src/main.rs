use async_std::{net::TcpListener, task};
use dotenv::dotenv;
use futures::{pin_mut, select, FutureExt};
use multicast_discovery::{create_multicast_socket, receive_multicast};
use std::{env, io, net::SocketAddrV4};

mod multicast_discovery;

async fn run_server() -> io::Result<()> {
    let server_addr: SocketAddrV4 = env::var("SERVER_ADDR")
        .expect("SERVER_ADDR must be set")
        .parse()
        .expect("SERVER_ADDR must be valid");
    let multicast_addr: SocketAddrV4 = env::var("MULTICAST_ADDR")
        .expect("MULTICAST_ADDR must be set")
        .parse()
        .expect("MULTICAST_ADDR must be valid");

    let multicast_socket = create_multicast_socket(&multicast_addr).await.unwrap();
    let tcp_listener = TcpListener::bind(server_addr).await?;

    loop {
        let multicast_receive_task = receive_multicast(&multicast_socket).fuse();
        pin_mut!(multicast_receive_task);

        select! (
            msg = multicast_receive_task => {
                if let Ok(msg) = msg {
                    println!("Received discovery message");
                    let _ = multicast_socket.send_to(tcp_listener.local_addr().unwrap().to_string().as_bytes(), multicast_addr.ip().to_string() + ":" + &msg).await;
                }
            }
        )
    }
}

fn main() -> io::Result<()> {
    dotenv().ok();

    task::block_on(async {
        run_server().await?;
        Ok(())
    })
}
