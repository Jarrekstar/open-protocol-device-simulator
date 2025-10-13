use crate::events::{EventBroadcaster, SimulatorEvent};
use crate::handler::data::TighteningResult;
use crate::state::DeviceState;
use axum::{
    extract::State as AxumState,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Shared state for HTTP server
#[derive(Clone)]
struct ServerState {
    device_state: Arc<RwLock<DeviceState>>,
    broadcaster: EventBroadcaster,
}

/// Start the HTTP server for state inspection and simulation control
pub async fn start_http_server(state: Arc<RwLock<DeviceState>>, broadcaster: EventBroadcaster) {
    let server_state = ServerState {
        device_state: state,
        broadcaster,
    };

    let app = Router::new()
        .route("/state", get(get_state))
        .route("/simulate/tightening", post(simulate_tightening))
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081")
        .await
        .expect("Failed to bind HTTP server to port 8081");

    println!("HTTP state server listening on http://0.0.0.0:8081");
    println!("Endpoints:");
    println!("  GET  /state                  - View device state");
    println!("  POST /simulate/tightening    - Simulate a tightening operation");

    axum::serve(listener, app)
        .await
        .expect("HTTP server failed");
}

/// Handler for GET /state endpoint
async fn get_state(
    AxumState(server_state): AxumState<ServerState>,
) -> Json<DeviceState> {
    let state = server_state.device_state.read().unwrap();
    Json(state.clone())
}

#[derive(Deserialize)]
struct TighteningRequest {
    #[serde(default = "default_torque")]
    torque: f64,
    #[serde(default = "default_angle")]
    angle: f64,
    #[serde(default = "default_status")]
    ok: bool,
}

fn default_torque() -> f64 {
    12.5
}
fn default_angle() -> f64 {
    40.0
}
fn default_status() -> bool {
    true
}

#[derive(Serialize)]
struct TighteningResponse {
    success: bool,
    message: String,
    batch_counter: u32,
    subscribers: usize,
}

/// Handler for POST /simulate/tightening endpoint
/// Simulates a tightening operation and broadcasts to subscribed clients
async fn simulate_tightening(
    AxumState(server_state): AxumState<ServerState>,
    Json(payload): Json<TighteningRequest>,
) -> impl IntoResponse {
    let (result, batch_counter, batch_completed) = {
        let mut state = server_state.device_state.write().unwrap();

        // Add tightening to batch manager
        let info = state.batch_manager.add_tightening(payload.ok);

        // Determine batch_status for MID 0061 param 22
        // None → 2 (not finished), Some(true) → 1 (OK), Some(false) → 0 (NOK)
        let batch_status = match info.batch_status {
            crate::batch_manager::BatchStatus::NotFinished => None,
            crate::batch_manager::BatchStatus::CompletedOk => Some(true),
            crate::batch_manager::BatchStatus::CompletedNok => Some(false),
        };

        // Build tightening result from current state
        let result = TighteningResult {
            cell_id: state.cell_id,
            channel_id: state.channel_id,
            controller_name: state.controller_name.clone(),
            vin_number: state.vehicle_id.clone(),
            job_id: state.current_job_id.unwrap_or(1),
            pset_id: state.current_pset_id.unwrap_or(1),
            batch_size: state.batch_manager.target_size(),
            batch_counter: info.counter,
            tightening_status: payload.ok,
            torque_status: payload.ok,
            angle_status: payload.ok,
            torque_min: 10.0,
            torque_max: 15.0,
            torque_target: 12.5,
            torque: payload.torque,
            angle_min: 30.0,
            angle_max: 50.0,
            angle_target: 40.0,
            angle: payload.angle,
            timestamp: chrono::Local::now().format("%Y-%m-%d:%H:%M:%S").to_string(),
            last_pset_change: None,
            batch_status,
            tightening_id: Some(info.tightening_id),
        };

        let batch_completed = state.batch_manager.is_complete();

        // Auto-reset batch if completed
        if batch_completed {
            state.batch_manager.reset();
        }

        (result, info.counter, batch_completed)
    };

    // Broadcast the tightening event to all TCP clients
    let event = SimulatorEvent::TighteningCompleted { result };
    let subscribers = server_state.broadcaster.receiver_count();

    let tightening_result = server_state.broadcaster.send(event);

    // If batch completed, emit batch completion event
    if batch_completed {
        let batch_event = SimulatorEvent::BatchCompleted { total: batch_counter };
        let _ = server_state.broadcaster.send(batch_event);
        println!("Batch completed with {} tightenings", batch_counter);
    }

    match tightening_result {
        Ok(_) => {
            println!("Tightening event broadcast to {} subscribers", subscribers);
            (
                StatusCode::OK,
                Json(TighteningResponse {
                    success: true,
                    message: format!("Tightening result broadcast to {} TCP client(s)", subscribers),
                    batch_counter,
                    subscribers,
                }),
            )
        }
        Err(e) => {
            eprintln!("Failed to broadcast event: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TighteningResponse {
                    success: false,
                    message: "Failed to broadcast tightening event".to_string(),
                    batch_counter,
                    subscribers: 0,
                }),
            )
        }
    }
}
