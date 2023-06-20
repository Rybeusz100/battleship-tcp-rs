use crate::session::handle_client;
use async_std::prelude::*;
use async_std::{net::TcpListener, task};
use dotenv::dotenv;
use log::warn;
use manager::start_manager;
use multicast_discovery::start_multicast_discovery;
use std::sync::atomic::{AtomicU32, Ordering};
use std::{env, io, net::SocketAddrV4};

#[cfg(all(target_os = "linux", feature = "daemonize"))]
use {daemonize::Daemonize, std::fs::File};

mod game;
mod manager;
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

    let manager_tx = start_manager();

    while let Some(Ok(stream)) = tcp_listener.incoming().next().await {
        if CONNECTIONS_COUNT.load(Ordering::SeqCst) < max_connections {
            handle_client(stream, manager_tx.clone());
        } else {
            warn!("Connection limit reached");
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    #[cfg(all(target_os = "linux", feature = "daemonize"))]
    {
        let stdout = File::create("/tmp/battleship.out").unwrap();
        let stderr = File::create("/tmp/battleship.err").unwrap();

        Daemonize::new()
            .stdout(stdout)
            .stderr(stderr)
            .start()
            .unwrap();
    }

    task::block_on(async {
        run_server().await?;
        Ok(())
    })
}
