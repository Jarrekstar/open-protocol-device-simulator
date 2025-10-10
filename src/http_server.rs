use crate::state::DeviceState;
use axum::{
    extract::State as AxumState,
    response::Json,
    routing::get,
    Router,
};
use std::sync::{Arc, RwLock};

/// Start the HTTP server for state inspection
pub async fn start_http_server(state: Arc<RwLock<DeviceState>>) {
    let app = Router::new()
        .route("/state", get(get_state))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081")
        .await
        .expect("Failed to bind HTTP server to port 8081");

    println!("HTTP state server listening on http://0.0.0.0:8081");
    println!("Query state with: curl http://localhost:8081/state");

    axum::serve(listener, app)
        .await
        .expect("HTTP server failed");
}

/// Handler for GET /state endpoint
async fn get_state(
    AxumState(state): AxumState<Arc<RwLock<DeviceState>>>,
) -> Json<DeviceState> {
    let state = state.read().unwrap();
    Json(state.clone())
}
