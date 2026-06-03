use std::{io, sync::Arc};

use axum::{
    Router as AxumRouter,
    body::Body,
    extract::{Request, State},
    response::{IntoResponse, Response},
    routing::get,
};
use hyper::{StatusCode, Uri};
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};

use crate::Peer;

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[derive(Clone)]
pub struct Router {
    listen_port: u16,
    pub client: Client,
    pub peers: Arc<Vec<Peer>>,
}

impl Router {
    pub fn new(listen_port: u16) -> Self {
        let client: Client =
            hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
                .build(HttpConnector::new());
        Self {
            listen_port,
            client,
            peers: Arc::new(Vec::new()),
        }
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
        let listen_addr = format!("localhost:{}", self.listen_port);
        let listener = tokio::net::TcpListener::bind(&listen_addr).await?;

        tracing::info!("Router listening on {listen_addr}");
        axum::serve(listener, app).await
    }

    pub fn select_peer(&self) -> Option<&Peer> {
        self.peers.first()
    }
}

async fn handler(State(state): State<Router>, mut req: Request) -> Result<Response, StatusCode> {
    if let Some(peer) = state.select_peer() {
        let path = req.uri().path();
        let path_query = req
            .uri()
            .path_and_query()
            .map(|p| p.as_str())
            .unwrap_or(path);
        let uri = format!("http://localhost:{}{path_query}", peer.port);

        tracing::info!("Forwarding request to {}", &uri);
        *req.uri_mut() = Uri::try_from(uri).unwrap();

        return Ok(state
            .client
            .request(req)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?
            .into_response());
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}
