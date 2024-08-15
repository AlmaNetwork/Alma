use anyhow::Result;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::task;

mod config;
mod server;
mod utils;
mod webrtc;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = config::get_cli_matches();
    let (mode, port, remote_address) = config::parse_cli_args(&matches)?;

    let addr = SocketAddr::from_str(&format!("127.0.0.1:{}", port))?;
    utils::set_remote_address(remote_address.clone()).await;

    let server_task = task::spawn(async move {
        if let Err(e) = server::start_server(addr).await {
            eprintln!("Failed to start server: {}", e);
        }
    });

    let peer_connection = webrtc::peer_connection::create_peer_connection().await?;

    if mode == "offer" {
        webrtc::peer_connection::handle_offer_mode(&peer_connection, &remote_address).await?;
    } else {
        webrtc::peer_connection::handle_answer_mode(&peer_connection).await?;
    }

    utils::wait_for_connection(&peer_connection).await;

    task::spawn(utils::auto_message_loop());
    utils::console_input_loop().await;

    peer_connection.close().await?;
    server_task.await.map_err(|e| anyhow::anyhow!("Server task error: {}", e))?;

    Ok(())
}