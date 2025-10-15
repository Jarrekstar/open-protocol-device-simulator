use crate::device_fsm::{DeviceFSM, DeviceFSMState, TighteningParams};
use crate::events::{EventBroadcaster, SimulatorEvent};
use crate::handler::data::TighteningResult;
use crate::state::DeviceState;
use axum::{
    Router,
    extract::State as AxumState,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Shared state for HTTP server
#[derive(Clone)]
struct ServerState {
    device_state: Arc<RwLock<DeviceState>>,
    broadcaster: EventBroadcaster,
    auto_tightening_active: Arc<AtomicBool>,
}

/// Helper function to build a TighteningResult from device state and tightening info
fn build_tightening_result(
    state: &DeviceState,
    info: &crate::batch_manager::TighteningInfo,
    torque: f64,
    angle: f64,
    tightening_ok: bool,
    torque_ok: bool,
    angle_ok: bool,
) -> TighteningResult {
    let batch_status = match info.batch_status {
        crate::batch_manager::BatchStatus::NotFinished => None,
        crate::batch_manager::BatchStatus::CompletedOk => Some(true),
        crate::batch_manager::BatchStatus::CompletedNok => Some(false),
        crate::batch_manager::BatchStatus::NotUsed => None,
    };

    TighteningResult {
        cell_id: state.cell_id,
        channel_id: state.channel_id,
        controller_name: state.controller_name.clone(),
        vin_number: state.vehicle_id.clone(),
        job_id: state.current_job_id.unwrap_or(1),
        pset_id: state.current_pset_id.unwrap_or(1),
        batch_size: state.tightening_tracker.batch_size(),
        batch_counter: info.counter,
        tightening_status: tightening_ok,
        torque_status: torque_ok,
        angle_status: angle_ok,
        torque_min: 10.0,
        torque_max: 15.0,
        torque_target: 12.5,
        torque,
        angle_min: 30.0,
        angle_max: 50.0,
        angle_target: 40.0,
        angle,
        timestamp: chrono::Local::now().format("%Y-%m-%d:%H:%M:%S").to_string(),
        last_pset_change: None,
        batch_status,
        tightening_id: Some(info.tightening_id),
    }
}

/// Start the HTTP server for state inspection and simulation control
pub async fn start_http_server(state: Arc<RwLock<DeviceState>>, broadcaster: EventBroadcaster) {
    let server_state = ServerState {
        device_state: state,
        broadcaster,
        auto_tightening_active: Arc::new(AtomicBool::new(false)),
    };

    let app = Router::new()
        .route("/state", get(get_state))
        .route("/simulate/tightening", post(simulate_tightening))
        .route("/auto-tightening/start", post(start_auto_tightening))
        .route("/auto-tightening/stop", post(stop_auto_tightening))
        .route("/auto-tightening/status", get(get_auto_tightening_status))
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081")
        .await
        .expect("Failed to bind HTTP server to port 8081");

    println!("HTTP state server listening on http://0.0.0.0:8081");
    println!("Endpoints:");
    println!("  GET  /state                       - View device state");
    println!("  POST /simulate/tightening         - Simulate a single tightening operation");
    println!(
        "  POST /auto-tightening/start       - Start automated tightening simulation (continuous)"
    );
    println!("  POST /auto-tightening/stop        - Stop automated tightening simulation");
    println!("  GET  /auto-tightening/status      - Get auto-tightening status");

    axum::serve(listener, app)
        .await
        .expect("HTTP server failed");
}

/// Handler for GET /state endpoint
async fn get_state(AxumState(server_state): AxumState<ServerState>) -> Json<DeviceState> {
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

        // Add tightening to tracker
        let info = state.tightening_tracker.add_tightening(payload.ok);

        // Build tightening result from current state
        let result = build_tightening_result(
            &state,
            &info,
            payload.torque,
            payload.angle,
            payload.ok,
            payload.ok,
            payload.ok,
        );

        let batch_completed = state.tightening_tracker.is_complete();

        // Note: Batch is NOT auto-reset here - integrator must send new batch config (MID 0019)

        (result, info.counter, batch_completed)
    };

    // Broadcast the tightening event to all TCP clients
    let event = SimulatorEvent::TighteningCompleted { result };
    let subscribers = server_state.broadcaster.receiver_count();

    let tightening_result = server_state.broadcaster.send(event);

    // If batch completed, emit batch completion event
    if batch_completed {
        let batch_event = SimulatorEvent::BatchCompleted {
            total: batch_counter,
        };
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
                    message: format!(
                        "Tightening result broadcast to {} TCP client(s)",
                        subscribers
                    ),
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

// ============================================================================
// Automated Tightening Simulation
// ============================================================================

#[derive(Deserialize)]
struct AutoTighteningRequest {
    /// Time between tightening cycles in milliseconds
    #[serde(default = "default_interval")]
    interval_ms: u64,
    /// Duration of each tightening operation in milliseconds
    #[serde(default = "default_duration")]
    duration_ms: u64,
    /// Probability of failure (0.0 = never fail, 1.0 = always fail)
    #[serde(default = "default_failure_rate")]
    failure_rate: f64,
}

fn default_interval() -> u64 {
    3000
} // 3 seconds between cycles
fn default_duration() -> u64 {
    1500
} // 1.5 second tightening
fn default_failure_rate() -> f64 {
    0.1
} // 10% failure rate

#[derive(Serialize)]
struct AutoTighteningResponse {
    success: bool,
    message: String,
    duration_ms: u64,
    interval_ms: u64,
}

/// Handler for POST /auto-tightening/start endpoint
/// Starts an automated tightening simulation in the background (continuous mode)
async fn start_auto_tightening(
    AxumState(server_state): AxumState<ServerState>,
    Json(payload): Json<AutoTighteningRequest>,
) -> impl IntoResponse {
    // Check if auto-tightening is already running
    if server_state.auto_tightening_active.load(Ordering::Relaxed) {
        return (
            StatusCode::CONFLICT,
            Json(AutoTighteningResponse {
                success: false,
                message: "Auto-tightening already running. Stop it first.".to_string(),
                duration_ms: 0,
                interval_ms: 0,
            }),
        );
    }

    let interval_ms = payload.interval_ms;
    let duration_ms = payload.duration_ms;
    let failure_rate = payload.failure_rate.clamp(0.0, 1.0);

    // Clone state for background task
    let state = Arc::clone(&server_state.device_state);
    let broadcaster = server_state.broadcaster.clone();
    let auto_active = Arc::clone(&server_state.auto_tightening_active);

    // Set active flag
    auto_active.store(true, Ordering::Relaxed);

    // Spawn background task
    tokio::spawn(async move {
        println!("Starting automated tightening (continuous mode)");

        let mut cycle = 0u64;
        while auto_active.load(Ordering::Relaxed) {
            // Check if tool is enabled
            let tool_enabled = {
                let s = state.read().unwrap();
                s.tool_enabled
            };

            if !tool_enabled {
                println!("Auto-tightening stopped: tool disabled");
                break;
            }

            // Check if we should wait for new configuration
            // In batch mode: waits when batch is complete
            // In single mode: never waits (integrator controls via tool enable/disable)
            let (should_wait, remaining) = {
                let s = state.read().unwrap();
                (
                    s.tightening_tracker.should_wait_for_config(),
                    s.tightening_tracker.remaining_work(),
                )
            };

            if should_wait {
                // Batch complete - wait for integrator to send new batch config (MID 0019)
                tokio::time::sleep(Duration::from_millis(interval_ms)).await;
                continue;
            }

            // Log remaining work (only meaningful in batch mode)
            if let Some(remaining_bolts) = remaining {
                if remaining_bolts == 0 {
                    tokio::time::sleep(Duration::from_millis(interval_ms)).await;
                    continue;
                }
            }

            // ================================================================
            // Phase 1: IDLE → TIGHTENING
            // ================================================================

            let params = TighteningParams::default_test();

            // Update state to reflect tightening in progress
            {
                let mut s = state.write().unwrap();
                let fsm = DeviceFSM::new().start_tightening(params.clone());
                s.device_fsm_state = DeviceFSMState::tightening(&fsm);
            }

            cycle += 1;
            if let Some(remaining_bolts) = remaining {
                println!(
                    "Cycle {}: Tightening started (remaining bolts: {})",
                    cycle, remaining_bolts
                );
            } else {
                println!("Cycle {}: Tightening started (single mode)", cycle);
            }

            // ================================================================
            // Phase 2: Simulate tightening duration
            // ================================================================

            tokio::time::sleep(Duration::from_millis(duration_ms)).await;

            // ================================================================
            // Phase 3: TIGHTENING → EVALUATING
            // ================================================================

            // Complete the tightening and get result
            let fsm = DeviceFSM::new().start_tightening(params);
            let fsm = fsm.complete();
            let outcome = fsm.result();

            // Apply failure rate (override natural variation)
            let seed = chrono::Local::now().timestamp_micros() as u64;
            let random_value = (seed % 100) as f64 / 100.0;
            let final_ok = if random_value < failure_rate {
                false // Force NOK based on failure rate
            } else {
                outcome.ok // Use natural OK/NOK from FSM
            };

            // Update state to evaluating
            {
                let mut s = state.write().unwrap();
                s.device_fsm_state = DeviceFSMState::evaluating(&fsm);
            }

            println!(
                "Cycle {}: Tightening complete - {} (torque: {:.2} Nm, angle: {:.1}°)",
                cycle,
                if final_ok { "OK" } else { "NOK" },
                outcome.actual_torque,
                outcome.actual_angle
            );

            // ================================================================
            // Phase 4: Add to batch and broadcast
            // ================================================================

            let (result, batch_counter, batch_completed) = {
                let mut s = state.write().unwrap();
                let info = s.tightening_tracker.add_tightening(final_ok);

                let result = build_tightening_result(
                    &s,
                    &info,
                    outcome.actual_torque,
                    outcome.actual_angle,
                    final_ok,
                    outcome.torque_ok,
                    outcome.angle_ok,
                );

                let batch_completed = s.tightening_tracker.is_complete();

                // Note: Batch is NOT auto-reset here - integrator must send new batch config (MID 0019)

                (result, info.counter, batch_completed)
            };

            // Broadcast to subscribed TCP clients
            let event = SimulatorEvent::TighteningCompleted { result };
            let _ = broadcaster.send(event);

            if batch_completed {
                let batch_event = SimulatorEvent::BatchCompleted {
                    total: batch_counter,
                };
                let _ = broadcaster.send(batch_event);
                println!("Batch completed with {} tightenings", batch_counter);
            }

            // ================================================================
            // Phase 5: EVALUATING → IDLE
            // ================================================================

            {
                let mut s = state.write().unwrap();
                s.device_fsm_state = DeviceFSMState::idle();
            }

            // Wait before next cycle
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
        }

        // Reset active flag when loop exits
        auto_active.store(false, Ordering::Relaxed);
        println!("Automated tightening stopped");
    });

    (
        StatusCode::OK,
        Json(AutoTighteningResponse {
            success: true,
            message: "Auto-tightening started (continuous mode)".to_string(),
            duration_ms,
            interval_ms,
        }),
    )
}

/// Handler for POST /auto-tightening/stop endpoint
/// Stops the automated tightening simulation
async fn stop_auto_tightening(
    AxumState(server_state): AxumState<ServerState>,
) -> impl IntoResponse {
    let was_running = server_state
        .auto_tightening_active
        .swap(false, Ordering::Relaxed);

    if was_running {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Auto-tightening stopped"
            })),
        )
    } else {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Auto-tightening was not running"
            })),
        )
    }
}

/// Auto-tightening status response
#[derive(Serialize)]
struct AutoTighteningStatus {
    running: bool,
    counter: u32,
    target_size: u32,
    remaining_bolts: u32,
}

/// Handler for GET /auto-tightening/status endpoint
/// Returns the current status of auto-tightening
async fn get_auto_tightening_status(
    AxumState(server_state): AxumState<ServerState>,
) -> Json<AutoTighteningStatus> {
    let running = server_state.auto_tightening_active.load(Ordering::Relaxed);
    let state = server_state.device_state.read().unwrap();
    let counter = state.tightening_tracker.counter();
    let target = state.tightening_tracker.batch_size();

    Json(AutoTighteningStatus {
        running,
        counter,
        target_size: target,
        remaining_bolts: target.saturating_sub(counter),
    })
}
