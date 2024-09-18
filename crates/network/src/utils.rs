use anyhow::{Result, anyhow};
use hyper::{Body, Client, Method, Request};
use std::io::{self, BufRead};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;

lazy_static::lazy_static! {
    static ref PEER_CONNECTION: Arc<Mutex<Option<Arc<RTCPeerConnection>>>> = Arc::new(Mutex::new(None));
    static ref REMOTE_ADDRESS: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref DATA_CHANNEL: Arc<Mutex<Option<Arc<RTCDataChannel>>>> = Arc::new(Mutex::new(None));
    static ref PENDING_CANDIDATES: Arc<Mutex<Vec<RTCIceCandidateInit>>> = Arc::new(Mutex::new(vec![]));
}

pub async fn set_peer_connection(pc: Arc<RTCPeerConnection>) {
    let mut peer_connection = PEER_CONNECTION.lock().await;
    *peer_connection = Some(pc);
}

pub async fn get_peer_connection() -> Option<Arc<RTCPeerConnection>> {
    let peer_connection = PEER_CONNECTION.lock().await;
    peer_connection.clone()
}

pub async fn set_remote_address(addr: String) {
    let mut ra = REMOTE_ADDRESS.lock().await;
    *ra = addr;
}

pub async fn get_remote_address() -> String {
    let ra = REMOTE_ADDRESS.lock().await;
    ra.clone()
}

pub async fn set_data_channel(dc: Arc<RTCDataChannel>) {
    let mut dcm = DATA_CHANNEL.lock().await;
    *dcm = Some(dc);
}

pub async fn get_pending_candidates() -> Vec<RTCIceCandidateInit> {
    let candidates = PENDING_CANDIDATES.lock().await;
    candidates.clone()
}

pub async fn set_pending_candidates(candidates: Vec<RTCIceCandidateInit>) {
    let mut pending_candidates = PENDING_CANDIDATES.lock().await;
    *pending_candidates = candidates;
}

pub async fn add_pending_candidate(candidate: RTCIceCandidateInit) {
    let mut candidates = PENDING_CANDIDATES.lock().await;
    candidates.push(candidate);
}

pub async fn send_request(addr: &str, path: &str, payload: String) -> Result<()> {
    let max_retries = 5;
    let mut retry_count = 0;
    let retry_delay = Duration::from_secs(1);

    while retry_count < max_retries {
        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("http://{}/{}", addr, path))
            .header("content-type", "application/json; charset=utf-8")
            .body(Body::from(payload.clone()))?;

        println!("Sending request to http://{}/{}", addr, path);
        match Client::new().request(req).await {
            Ok(resp) => {
                println!("Received response with status: {}", resp.status());
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error sending request (attempt {}): {}", retry_count + 1, e);
                retry_count += 1;
                if retry_count < max_retries {
                    println!("Retrying in {} seconds...", retry_delay.as_secs());
                    sleep(retry_delay).await;
                }
            }
        }
    }

    Err(anyhow!("Failed to send request after {} attempts", max_retries))
}

pub async fn send_message(message: String) -> Result<()> {
    let dc = DATA_CHANNEL.lock().await;
    if let Some(channel) = dc.as_ref() {
        channel.send_text(message).await?;
        Ok(())
    } else {
        Err(anyhow!("Data channel not initialized"))
    }
}

pub async fn console_input_loop() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    loop {
        println!("Enter a message to send (or 'quit' to exit):");
        if let Some(Ok(line)) = lines.next() {
            if line.trim().to_lowercase() == "quit" {
                break;
            }
            match send_message(line).await {
                Ok(_) => println!("Message sent successfully"),
                Err(e) => eprintln!("Failed to send message: {}", e),
            }
        }
    }
}

pub async fn auto_message_loop() {
    loop {
        sleep(Duration::from_secs(5)).await;
        let message = webrtc::peer_connection::math_rand_alpha(3);
        println!("Sending auto message: '{}'", message);
        if let Err(e) = send_message(message).await {
            eprintln!("Failed to send auto message: {}", e);
        }
    }
}

pub async fn wait_for_connection(peer_connection: &Arc<RTCPeerConnection>) {
    while peer_connection.connection_state() != RTCPeerConnectionState::Connected {
        sleep(Duration::from_secs(1)).await;
    }
    println!("Connection established! You can now start sending messages.");
}