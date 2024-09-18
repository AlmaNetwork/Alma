use anyhow::Result;
use std::sync::Arc;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};

use crate::utils;
use crate::webrtc::data_channel;

pub async fn create_peer_connection() -> Result<Arc<RTCPeerConnection>> {
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let api = APIBuilder::new().build();
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    setup_ice_candidate_handler(&peer_connection);
    utils::set_peer_connection(Arc::clone(&peer_connection)).await;

    Ok(peer_connection)
}

pub async fn handle_offer_mode(peer_connection: &Arc<RTCPeerConnection>, remote_addr: &str) -> Result<()> {
    let data_channel = peer_connection.create_data_channel("data", None).await?;
    data_channel::setup_data_channel_handlers(Arc::clone(&data_channel)).await;

    let offer = peer_connection.create_offer(None).await?;
    peer_connection.set_local_description(offer.clone()).await?;

    let payload = serde_json::to_string(&offer)?;
    utils::send_request(remote_addr, "sdp", payload).await?;

    Ok(())
}

pub async fn handle_answer_mode(peer_connection: &Arc<RTCPeerConnection>) -> Result<()> {
    peer_connection.on_data_channel(Box::new(move |d: Arc<webrtc::data_channel::RTCDataChannel>| {
        println!("New DataChannel {}", d.label());
        let d_clone = Arc::clone(&d);
        Box::pin(async move {
            data_channel::setup_data_channel_handlers(d_clone).await;
        })
    }));

    Ok(())
}

fn setup_ice_candidate_handler(peer_connection: &Arc<RTCPeerConnection>) {
    let pc = Arc::downgrade(&peer_connection);
    peer_connection.on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
        let pc = pc.clone();
        Box::pin(async move {
            if let Some(c) = c {
                if let Some(pc) = pc.upgrade() {
                    let remote_addr = utils::get_remote_address().await;
                    if pc.remote_description().await.is_some() {
                        let payload = serde_json::to_string(&c.to_json().unwrap()).unwrap();
                        if let Err(err) = utils::send_request(&remote_addr, "candidate", payload).await {
                            eprintln!("Error signaling candidate: {}", err);
                        }
                    } else {
                        utils::add_pending_candidate(RTCIceCandidateInit {
                            candidate: c.to_string(),
                            ..Default::default()
                        }).await;
                    }
                }
            }
        })
    }));
}