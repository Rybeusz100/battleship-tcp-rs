use crate::session::handle_client;
use async_std::prelude::*;
use async_std::{net::TcpListener, task};
use dotenv::dotenv;
use multicast_discovery::start_multicast_discovery;
use std::sync::atomic::{AtomicU32, Ordering};
use std::{env, io, net::SocketAddrV4};

mod game;
mod multicast_discovery;
mod session;

static CONNECTIONS_COUNT: AtomicU32 = AtomicU32::new(0);

async fn run_server() -> io::Result<()> {
    let server_addr: SocketAddrV4 = env::var("SERVER_ADDR")
        .expect("SERVER_ADDR must be set")
        .parse()
        .expect("SERVER_ADDR must be valid");
    let multicast_addr: SocketAddrV4 = env::var("MULTICAST_ADDR")
        .expect("MULTICAST_ADDR must be set")
        .parse()
        .expect("MULTICAST_ADDR must be valid");
    let max_connections: u32 = env::var("MAX_CONNECTIONS")
        .expect("MAX_CONNECTIONS must be set")
        .parse()
        .expect("MAX_CONNECTIONS must be valid");

    let tcp_listener = TcpListener::bind(server_addr).await?;

    start_multicast_discovery(
        multicast_addr,
        tcp_listener.local_addr().unwrap().to_string(),
    );

    while let Some(Ok(stream)) = tcp_listener.incoming().next().await {
        if CONNECTIONS_COUNT.load(Ordering::SeqCst) < max_connections {
            handle_client(stream);
        } else {
            println!("Connection limit reached");
        }
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
