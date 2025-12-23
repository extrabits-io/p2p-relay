use std::io;

use axum::{
    Router, body::Body, extract::{Request, State}, response::{IntoResponse, Response}, routing::get
};
use hyper::{StatusCode, Uri};
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};

use crate::{config::ProxyConfig, server::Peer};

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[derive(Clone)]
struct ProxyState {
    pub client: Client,
    pub peers: Vec<Peer>,
}

pub async fn start(config: &ProxyConfig, peers: &Vec<Peer>) -> anyhow::Result<(), io::Error> {
    let client: Client =
        hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
            .build(HttpConnector::new());
    let state = ProxyState {
        client,
        peers: peers.clone(),
    };
    let app = Router::new()
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
        .with_state(state);
    let listen_addr = format!("{}:{}", config.listen_url, config.listen_port);
    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;

    log::info!("Proxy listening on {listen_addr}");
    axum::serve(listener, app).await
}

async fn handler(State(state): State<ProxyState>, mut req: Request) -> Result<Response, StatusCode> {
    if let Some(peer) = state.peers.first() {
        let path = req.uri().path();
        let path_query = req
            .uri()
            .path_and_query()
            .map(|p| p.as_str())
            .unwrap_or(path);
        let uri = format!("http://{}:{}{path_query}", &peer.address.ip_address, peer.port);

        log::info!("Forwarding request to {}", &uri);
        *req.uri_mut() = Uri::try_from(uri).unwrap();

        return Ok(state.client
            .request(req)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?
            .into_response());
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}
