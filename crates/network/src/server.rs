use anyhow::{Result, anyhow};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::net::SocketAddr;
use std::sync::Arc;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

use crate::utils::{send_request, get_remote_address, get_pending_candidates, set_pending_candidates};

pub async fn start_server(addr: SocketAddr) -> Result<()> {
    let service = make_service_fn(|_| async { 
        Ok::<_, hyper::Error>(service_fn(|req| remote_handler(req)))
    });
    let server = Server::try_bind(&addr)
        .map_err(|e| anyhow!("Failed to bind to {}: {}", addr, e))?
        .serve(service);
    println!("Server listening on http://{}", addr);
    server.await.map_err(|e| anyhow!("Server error: {}", e))
}

async fn remote_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let result: Result<Response<Body>, anyhow::Error> = async {
        let peer_connection = crate::utils::get_peer_connection().await
            .ok_or_else(|| anyhow!("PeerConnection not initialized"))?;

        match (req.method(), req.uri().path()) {
            (&Method::POST, "/candidate") => handle_candidate(peer_connection, req).await,
            (&Method::POST, "/sdp") => handle_sdp(peer_connection, req).await,
            _ => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty())?)
        }
    }.await;

    match result {
        Ok(response) => Ok(response),
        Err(e) => {
            eprintln!("Error in remote_handler: {}", e);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Error: {}", e)))
                .unwrap())
        }
    }
}

async fn handle_candidate(pc: Arc<RTCPeerConnection>, req: Request<Body>) -> Result<Response<Body>> {
    println!("Received ICE candidate");
    let candidate = std::str::from_utf8(&hyper::body::to_bytes(req.into_body()).await?)
        .map_err(|e| anyhow!("Failed to parse candidate: {}", e))?
        .to_owned();

    println!("Raw ICE candidate: {}", candidate);

    let ice_candidate: RTCIceCandidateInit = serde_json::from_str(&candidate)
        .map_err(|e| anyhow!("Failed to deserialize ICE candidate: {}", e))?;

    if pc.remote_description().await.is_some() {
        println!("Adding ICE candidate");
        pc.add_ice_candidate(ice_candidate)
            .await
            .map_err(|e| anyhow!("Failed to add ICE candidate: {}", e))?;
        println!("ICE candidate added successfully");
    } else {
        println!("Storing ICE candidate for later");
        let mut candidates = get_pending_candidates().await;
        candidates.push(ice_candidate);
        set_pending_candidates(candidates).await;
    }

    Ok(Response::new(Body::empty()))
}

async fn handle_sdp(pc: Arc<RTCPeerConnection>, req: Request<Body>) -> Result<Response<Body>> {
    println!("Received SDP");
    let sdp_str = std::str::from_utf8(&hyper::body::to_bytes(req.into_body()).await?)
        .map_err(|e| anyhow!("Failed to parse SDP: {}", e))?
        .to_owned();
    let sdp: RTCSessionDescription = serde_json::from_str(&sdp_str)
        .map_err(|e| anyhow!("Failed to deserialize SDP: {}", e))?;

    println!("Setting remote description");
    pc.set_remote_description(sdp)
        .await
        .map_err(|e| anyhow!("Failed to set remote description: {}", e))?;
    println!("Remote description set successfully");

    // Apply any pending candidates
    let mut candidates = get_pending_candidates().await;
    println!("Applying {} pending ICE candidates", candidates.len());
    for candidate in candidates.drain(..) {
        match pc.add_ice_candidate(candidate).await {
            Ok(_) => println!("Pending ICE candidate added successfully"),
            Err(e) => eprintln!("Failed to add pending ICE candidate: {}", e),
        }
    }
    set_pending_candidates(candidates).await;
    println!("All pending ICE candidates processed");

    if pc.remote_description().await.is_some() {
        println!("Creating answer");
        let answer = pc.create_answer(None).await
            .map_err(|e| anyhow!("Failed to create answer: {}", e))?;
        println!("Setting local description");
        pc.set_local_description(answer.clone()).await
            .map_err(|e| anyhow!("Failed to set local description: {}", e))?;
        println!("Local description set successfully");

        let addr = get_remote_address().await;
        if !addr.is_empty() {
            println!("Sending answer to {}", addr);
            let payload = serde_json::to_string(&answer)
                .map_err(|e| anyhow!("Failed to serialize answer: {}", e))?;
            send_request(&addr, "sdp", payload).await
                .map_err(|e| anyhow!("Failed to send answer: {}", e))?;
            println!("Answer sent successfully");
        } else {
            println!("Remote address is not set. Skipping answer sending.");
        }
    }

    Ok(Response::new(Body::empty()))
}