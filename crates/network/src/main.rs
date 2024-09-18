mod config;
mod server;
mod utils;
mod webrtc;

use anyhow::Result;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::task;
use clap::Parser;
use tokio::time::{timeout, Duration};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long)]
    port: Option<String>,

    #[arg(long)]
    remote_address: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = if let Some(config_path) = cli.config {
        config::load_config(&config_path)?
    } else {
        config::NetworkConfig::default()
    };

    // Command line arguments override config file and default values
    if let Some(port) = cli.port {
        config.port = port;
    }
    if let Some(remote_address) = cli.remote_address {
        config.remote_address = remote_address;
    }

    println!("Using configuration:");
    println!("Port: {}", config.port);
    println!("Remote Address: {}", config.remote_address);

    let local_addr = SocketAddr::from_str(&format!("127.0.0.1:{}", config.port))?;
    let remote_addr = SocketAddr::from_str(&config.remote_address)?;
    utils::set_remote_address(config.remote_address.clone()).await;

    let peer_connection = webrtc::peer_connection::create_peer_connection().await?;

    let server_task = task::spawn(server::start_server(local_addr));

    // Try to connect to the remote address
    let is_offer = match timeout(Duration::from_secs(5), tokio::net::TcpStream::connect(remote_addr)).await {
        Ok(Ok(_)) => {
            println!("Connected to remote. Operating in Offer mode.");
            true
        },
        _ => {
            println!("Failed to connect to remote. Operating in Answer mode and waiting for incoming connection.");
            false
        }
    };

    if is_offer {
        webrtc::peer_connection::handle_offer_mode(&peer_connection, &config.remote_address).await?;
    } else {
        webrtc::peer_connection::handle_answer_mode(&peer_connection).await?;
    }

    utils::wait_for_connection(&peer_connection).await;

    task::spawn(utils::auto_message_loop());
    utils::console_input_loop().await;

    peer_connection.close().await?;
    server_task.await??;

    Ok(())
}