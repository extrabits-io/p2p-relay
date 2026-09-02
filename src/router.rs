use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::{
    Router as AxumRouter,
    body::Body,
    extract::{Request, State},
    response::{IntoResponse, Response},
    routing::get,
};
use hyper::{StatusCode, Uri};
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use p2p_lib::shared::{PeerInfo, PeerKey};

use crate::Peer;

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[derive(Clone)]
pub struct Router {
    listen_port: u16,
    client: Client,
    peers: Arc<Mutex<HashMap<PeerKey, Peer>>>,
}

impl Router {
    pub fn add_peer(&self, peer: Peer) {
        self.peers.lock().unwrap().insert(peer.public_key, peer);
    }

    pub fn new(listen_port: u16) -> Self {
        let client: Client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
            .build(HttpConnector::new());
        Self {
            listen_port,
            client,
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn remove_peer(&self, public_key: PeerKey) {
        self.peers.lock().unwrap().remove(&public_key);
    }

    pub async fn start(self) -> anyhow::Result<(), io::Error> {
        let app = AxumRouter::new()
            .route(
                "/",
                get(handler)
                    .post(handler)
                    .put(handler)
                    .patch(handler)
                    .delete(handler)
                    .options(handler),
            )
            .route(
                "/{*path}",
                get(handler)
                    .post(handler)
                    .put(handler)
                    .patch(handler)
                    .delete(handler)
                    .options(handler),
            )
            .with_state(self.clone());
        let listen_addr = format!("0.0.0.0:{}", self.listen_port);
        let listener = tokio::net::TcpListener::bind(&listen_addr).await?;

        tracing::info!("router listening on {listen_addr}");
        axum::serve(listener, app).await
    }

    pub fn select_peer(&self) -> Option<PeerInfo> {
        let peers = self.peers.lock().unwrap();
        let mut best = Duration::MAX;
        let mut candidate = None;
        for peer in peers.values() {
            if let Some(latency) = peer.last_latency {
                if latency < best {
                    best = latency;
                    candidate = Some(peer);
                }
            } else {
                best = Duration::ZERO;
                candidate = Some(peer);
            }
        }
        candidate.map(|peer| (peer.public_key, peer.port))
    }

    pub fn update_latency(&self, key: PeerKey, latency: Duration) {
        if let Some(peer) = self.peers.lock().unwrap().get_mut(&key) {
            peer.last_latency = Some(latency);
        }
    }
}

async fn handler(State(state): State<Router>, mut req: Request) -> Result<Response, StatusCode> {
    if let Some((key, port)) = state.select_peer() {
        let path = req.uri().path();
        let path_query = req
            .uri()
            .path_and_query()
            .map(|p| p.as_str())
            .unwrap_or(path);
        let uri = format!("http://0.0.0.0:{port}{path_query}");

        tracing::info!("forwarding request to {}", &uri);
        *req.uri_mut() = Uri::try_from(uri).map_err(|e| {
            tracing::warn!("unable to construct URI: {e}");
            StatusCode::BAD_REQUEST
        })?;

        let start = Instant::now();
        let resp = state
            .client
            .request(req)
            .await
            .map_err(|e| {
                tracing::warn!("request failed:  {e}");
                StatusCode::BAD_REQUEST
            })?
            .into_response();

        let latency = if resp.status().is_server_error() {
            tracing::error!("peer {key} returned error: {}", resp.status());
            Duration::MAX
        } else {
            let dur = Instant::now().duration_since(start);
            tracing::info!("request completed in {:?}", dur);
            dur
        };
        state.update_latency(key, latency);

        return Ok(resp);
    }
    tracing::warn!("no peers available");
    Ok(StatusCode::NO_CONTENT.into_response())
}
