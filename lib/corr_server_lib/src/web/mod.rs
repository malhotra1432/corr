use warp::{Filter, Reply, Rejection};
use warp::{http::StatusCode};
use std::convert::Infallible;
use tokio::sync::{RwLock};
use std::collections::HashMap;
use ws::Clients;
use std::sync::Arc;

mod ws;
type Result<T> = std::result::Result<T, Rejection>;
pub async fn start_web_server(){
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
    let health_route = warp::path!("health").and_then(health_handler);
    let ws_route = warp::path!("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and_then(ws::handler);
    let routes = ws_route
        .or(health_route)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
pub async fn health_handler() -> Result<impl Reply> {
    println!("Called - health");
    Ok(StatusCode::OK)
}
fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}