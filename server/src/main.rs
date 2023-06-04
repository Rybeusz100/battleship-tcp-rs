use async_std::prelude::*;
use async_std::{net::TcpListener, task};
use dotenv::dotenv;
use multicast_discovery::start_multicast_discovery;
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

    let tcp_listener = TcpListener::bind(server_addr).await?;

    start_multicast_discovery(
        multicast_addr,
        tcp_listener.local_addr().unwrap().to_string(),
    );

    while let Some(_stream) = tcp_listener.incoming().next().await {
        println!("Incoming TCP connection");
    }

    Ok(())
}

fn main() -> io::Result<()> {
    dotenv().ok();

    task::block_on(async {
        run_server().await?;
        Ok(())
    })
}
