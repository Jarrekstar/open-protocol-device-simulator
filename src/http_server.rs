use crate::device_fsm::{DeviceFSM, DeviceFSMState, TighteningParams};
use crate::events::SimulatorEvent;
use crate::handler::data::TighteningResult;
use crate::multi_spindle::{MultiSpindleStatus, generate_multi_spindle_results};
use crate::observable_state::ObservableState;
use crate::pset::{self, SharedPsetRepository};
use crate::state::DeviceState;
use axum::{
    Router,
    extract::{
        Path,
        State as AxumState,
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{CorsLayer, Any};

/// Shared state for HTTP server
#[derive(Clone)]
pub struct ServerState {
    pub observable_state: ObservableState,
    pub auto_tightening_active: Arc<AtomicBool>,
    pub pset_repository: SharedPsetRepository,
}

/// Get TighteningParams from selected PSET, or default if no PSET selected
fn get_tightening_params(
    pset_id: Option<u32>,
    pset_repo: &SharedPsetRepository,
    duration_ms: u64,
) -> TighteningParams {
    if let Some(id) = pset_id {
        let repo = pset_repo.read().unwrap();
        if let Some(pset) = repo.get_by_id(id) {
            let target_torque = (pset.torque_min + pset.torque_max) / 2.0;
            let target_angle = (pset.angle_min + pset.angle_max) / 2.0;
            return TighteningParams {
                target_torque,
                torque_min: pset.torque_min,
                torque_max: pset.torque_max,
                target_angle,
                angle_min: pset.angle_min,
                angle_max: pset.angle_max,
                duration_ms,
            };
        }
    }

    // Fall back to default if no PSET selected
    TighteningParams::default_test()
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
    params: &TighteningParams,
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
        torque_min: params.torque_min,
        torque_max: params.torque_max,
        torque_target: params.target_torque,
        torque,
        angle_min: params.angle_min,
        angle_max: params.angle_max,
        angle_target: params.target_angle,
        angle,
        timestamp: chrono::Local::now().format("%Y-%m-%d:%H:%M:%S").to_string(),
        last_pset_change: None,
        batch_status,
        tightening_id: Some(info.tightening_id),
    }
}

/// Create the HTTP router with all endpoints configured
pub fn create_router(observable_state: ObservableState) -> Router {
    let pset_repository = crate::pset::create_sqlite_repository("simulator.db")
        .unwrap_or_else(|e| {
            eprintln!("Failed to create SQLite repository: {}. Falling back to in-memory.", e);
            crate::pset::create_default_repository()
        });

    let server_state = ServerState {
        observable_state,
        auto_tightening_active: Arc::new(AtomicBool::new(false)),
        pset_repository,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/state", get(get_state))
        .route("/simulate/tightening", post(simulate_tightening))
        .route("/auto-tightening/start", post(start_auto_tightening))
        .route("/auto-tightening/stop", post(stop_auto_tightening))
        .route("/auto-tightening/status", get(get_auto_tightening_status))
        .route("/config/multi-spindle", post(configure_multi_spindle))
        .route("/psets", get(get_psets).post(create_pset))
        .route("/psets/{id}", get(get_pset_by_id).put(update_pset).delete(delete_pset))
        .route("/psets/{id}/select", post(select_pset))
        .route("/ws/events", get(websocket_handler))
        .layer(cors)
        .with_state(server_state)
}

/// Start the HTTP server for state inspection and simulation control
pub async fn start_http_server(observable_state: ObservableState) {
    let app = create_router(observable_state);

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
    println!("  POST /config/multi-spindle        - Configure multi-spindle mode");
    println!("  GET  /ws/events                   - WebSocket event stream");

    axum::serve(listener, app)
        .await
        .expect("HTTP server failed");
}

/// Handler for GET /state endpoint
async fn get_state(AxumState(server_state): AxumState<ServerState>) -> Json<DeviceState> {
    let state = server_state.observable_state.read();
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
    Json(_payload): Json<TighteningRequest>,
) -> impl IntoResponse {
    // Get tightening params from selected PSET
    let params = {
        let state = server_state.observable_state.read();
        get_tightening_params(
            state.current_pset_id,
            &server_state.pset_repository,
            500, // duration_ms for simulation
        )
    };

    println!(
        "Simulating tightening with params: Torque {:.1}-{:.1} Nm (target: {:.1}), Angle {:.1}-{:.1}° (target: {:.1})",
        params.torque_min, params.torque_max, params.target_torque,
        params.angle_min, params.angle_max, params.target_angle
    );

    // Run FSM simulation
    let fsm = DeviceFSM::new();
    let fsm = fsm.start_tightening(params.clone());
    tokio::time::sleep(Duration::from_millis(10)).await; // Brief simulation
    let fsm = fsm.complete();
    let outcome = fsm.result();

    println!(
        "Result: Torque={:.2} Nm ({}), Angle={:.1}° ({}), Overall: {}",
        outcome.actual_torque,
        if outcome.torque_ok { "OK" } else { "NOK" },
        outcome.actual_angle,
        if outcome.angle_ok { "OK" } else { "NOK" },
        if outcome.ok { "OK" } else { "NOK" }
    );

    let (result, batch_counter, batch_completed) = {
        let mut state = server_state.observable_state.write();

        // Add tightening to tracker
        let info = state.tightening_tracker.add_tightening(outcome.ok);

        // Build tightening result from device state
        let result = build_tightening_result(
            &state,
            &info,
            outcome.actual_torque,
            outcome.actual_angle,
            outcome.ok,
            outcome.torque_ok,
            outcome.angle_ok,
            &params,
        );

        let batch_completed = state.tightening_tracker.is_complete();

        // Note: Batch is NOT auto-reset here - integrator must send new batch config (MID 0019)

        (result, info.counter, batch_completed)
    };

    // Broadcast the tightening event to all TCP clients
    let event = SimulatorEvent::TighteningCompleted { result };
    server_state.observable_state.broadcast(event);

    // If batch completed, emit batch completion event
    if batch_completed {
        let batch_event = SimulatorEvent::BatchCompleted {
            total: batch_counter,
        };
        server_state.observable_state.broadcast(batch_event);
        println!("Batch completed with {} tightenings", batch_counter);
    }

    let subscribers = 0; // WebSocket subscribers (not tracked in current API)
    let tightening_result: Result<(), String> = Ok(());

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

    // Clone observable state for background task
    let observable_state = server_state.observable_state.clone();
    let auto_active = Arc::clone(&server_state.auto_tightening_active);
    let pset_repository = Arc::clone(&server_state.pset_repository);

    // Set active flag
    auto_active.store(true, Ordering::Relaxed);

    // Spawn background task
    tokio::spawn(async move {
        println!("Starting automated tightening (continuous mode)");

        let mut cycle = 0u64;
        while auto_active.load(Ordering::Relaxed) {
            // Check if tool is enabled
            let tool_enabled = {
                let s = observable_state.read();
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
                let s = observable_state.read();
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

            // Get params from selected PSET
            let params = {
                let s = observable_state.read();
                get_tightening_params(s.current_pset_id, &pset_repository, duration_ms)
            };

            // Update state to reflect tightening in progress
            {
                let mut s = observable_state.write();
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
            let fsm = DeviceFSM::new().start_tightening(params.clone());
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
                let mut s = observable_state.write();
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

            // Check if multi-spindle mode is enabled
            let (multi_spindle_enabled, multi_spindle_config) = {
                let s = observable_state.read();
                (
                    s.multi_spindle_config.enabled,
                    s.multi_spindle_config.clone(),
                )
            };

            if multi_spindle_enabled {
                // ============================================================
                // MULTI-SPINDLE PATH
                // ============================================================

                // Get result_id and pset_id before generating results
                let (result_id, pset_id) = {
                    let s = observable_state.read();
                    (
                        s.tightening_tracker.tightening_sequence() + 1, // Next sequence number
                        s.current_pset_id.unwrap_or(1),
                    )
                };

                println!(
                    "Cycle {}: Multi-spindle tightening - {} spindles (sync_id: {})",
                    cycle, multi_spindle_config.spindle_count, multi_spindle_config.sync_id
                );

                // Broadcast "Running" status (MID 0091)
                let running_status = MultiSpindleStatus::running(
                    multi_spindle_config.sync_id,
                    multi_spindle_config.spindle_count,
                );
                observable_state.broadcast(SimulatorEvent::MultiSpindleStatusCompleted {
                    status: running_status,
                });

                // Generate multi-spindle results
                let multi_result =
                    generate_multi_spindle_results(&multi_spindle_config, result_id, pset_id);

                // Log per-spindle results
                for spindle in &multi_result.spindle_results {
                    println!(
                        "  Spindle {}: {} (torque: {:.2} Nm, angle: {:.1}°)",
                        spindle.spindle_id,
                        if spindle.is_ok() { "OK" } else { "NOK" },
                        spindle.torque as f64 / 100.0,
                        spindle.angle as f64 / 10.0
                    );
                }

                // Determine overall status for tracker
                let overall_ok = multi_result.is_ok();

                // Broadcast multi-spindle result (MID 0101)
                observable_state.broadcast(SimulatorEvent::MultiSpindleResultCompleted {
                    result: multi_result,
                });

                // Broadcast "Completed" status (MID 0091)
                let completed_status = MultiSpindleStatus::completed(
                    multi_spindle_config.sync_id,
                    multi_spindle_config.spindle_count,
                );
                observable_state.broadcast(SimulatorEvent::MultiSpindleStatusCompleted {
                    status: completed_status,
                });

                // Update tracker with overall status
                let (batch_counter, batch_completed, target_size) = {
                    let mut s = observable_state.write();
                    let info = s.tightening_tracker.add_tightening(overall_ok);
                    let batch_completed = s.tightening_tracker.is_complete();
                    let target = s.tightening_tracker.batch_size();
                    (info.counter, batch_completed, target)
                };

                // Broadcast auto-tightening progress
                let is_running = auto_active.load(Ordering::Relaxed);
                observable_state.broadcast_auto_progress(batch_counter, target_size, is_running);

                if batch_completed {
                    let batch_event = SimulatorEvent::BatchCompleted {
                        total: batch_counter,
                    };
                    observable_state.broadcast(batch_event);
                    println!("Batch completed with {} tightenings", batch_counter);
                }
            } else {
                // ============================================================
                // SINGLE-SPINDLE PATH
                // ============================================================

                let (result, batch_counter, batch_completed, target_size) = {
                    let mut s = observable_state.write();
                    let info = s.tightening_tracker.add_tightening(final_ok);

                    let result = build_tightening_result(
                        &s,
                        &info,
                        outcome.actual_torque,
                        outcome.actual_angle,
                        final_ok,
                        outcome.torque_ok,
                        outcome.angle_ok,
                        &params,
                    );

                    let batch_completed = s.tightening_tracker.is_complete();
                    let target = s.tightening_tracker.batch_size();

                    // Note: Batch is NOT auto-reset here - integrator must send new batch config (MID 0019)

                    (result, info.counter, batch_completed, target)
                };

                // Broadcast to subscribed TCP clients
                let event = SimulatorEvent::TighteningCompleted { result };
                observable_state.broadcast(event);

                // Broadcast auto-tightening progress
                let is_running = auto_active.load(Ordering::Relaxed);
                observable_state.broadcast_auto_progress(batch_counter, target_size, is_running);

                if batch_completed {
                    let batch_event = SimulatorEvent::BatchCompleted {
                        total: batch_counter,
                    };
                    observable_state.broadcast(batch_event);
                    println!("Batch completed with {} tightenings", batch_counter);
                }
            }

            // ================================================================
            // Phase 5: EVALUATING → IDLE
            // ================================================================

            {
                let mut s = observable_state.write();
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
        // Broadcast the stopped status
        let (counter, target_size) = {
            let state = server_state.observable_state.read();
            let counter = state.tightening_tracker.counter();
            let target = state.tightening_tracker.batch_size();
            (counter, target)
        };

        server_state.observable_state.broadcast_auto_progress(counter, target_size, false);

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
/// Returns the status of auto-tightening
async fn get_auto_tightening_status(
    AxumState(server_state): AxumState<ServerState>,
) -> Json<AutoTighteningStatus> {
    let running = server_state.auto_tightening_active.load(Ordering::Relaxed);
    let state = server_state.observable_state.read();
    let counter = state.tightening_tracker.counter();
    let target = state.tightening_tracker.batch_size();

    Json(AutoTighteningStatus {
        running,
        counter,
        target_size: target,
        remaining_bolts: target.saturating_sub(counter),
    })
}

// ============================================================================
// Multi-Spindle Configuration
// ============================================================================

#[derive(Deserialize)]
struct MultiSpindleConfigRequest {
    /// Enable or disable multi-spindle mode
    enabled: bool,
    /// Number of spindles (2-16, only used if enabled=true)
    #[serde(default = "default_spindle_count")]
    spindle_count: u8,
    /// Sync tightening ID (only used if enabled=true)
    #[serde(default = "default_sync_id")]
    sync_id: u32,
}

fn default_spindle_count() -> u8 {
    2
}
fn default_sync_id() -> u32 {
    1
}

#[derive(Serialize)]
struct MultiSpindleConfigResponse {
    success: bool,
    message: String,
    enabled: bool,
    spindle_count: u8,
    sync_id: u32,
}

/// Handler for POST /config/multi-spindle endpoint
/// Configures multi-spindle mode (enable/disable)
async fn configure_multi_spindle(
    AxumState(server_state): AxumState<ServerState>,
    Json(payload): Json<MultiSpindleConfigRequest>,
) -> impl IntoResponse {
    if payload.enabled {
        // Enable multi-spindle mode
        match server_state
            .observable_state
            .enable_multi_spindle(payload.spindle_count, payload.sync_id)
        {
            Ok(_) => {
                println!(
                    "Multi-spindle mode enabled: {} spindles, sync_id={}",
                    payload.spindle_count, payload.sync_id
                );
                (
                    StatusCode::OK,
                    Json(MultiSpindleConfigResponse {
                        success: true,
                        message: format!(
                            "Multi-spindle mode enabled with {} spindles",
                            payload.spindle_count
                        ),
                        enabled: true,
                        spindle_count: payload.spindle_count,
                        sync_id: payload.sync_id,
                    }),
                )
            }
            Err(e) => {
                eprintln!("Failed to enable multi-spindle mode: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(MultiSpindleConfigResponse {
                        success: false,
                        message: format!("Failed to enable multi-spindle mode: {}", e),
                        enabled: false,
                        spindle_count: 1,
                        sync_id: 0,
                    }),
                )
            }
        }
    } else {
        // Disable multi-spindle mode
        server_state.observable_state.disable_multi_spindle();
        println!("Multi-spindle mode disabled");
        (
            StatusCode::OK,
            Json(MultiSpindleConfigResponse {
                success: true,
                message: "Multi-spindle mode disabled".to_string(),
                enabled: false,
                spindle_count: 1,
                sync_id: 0,
            }),
        )
    }
}

// ============================================================================
// WebSocket Event Stream
// ============================================================================

/// Handler for GET /ws/events endpoint
/// Upgrades the HTTP connection to WebSocket and streams events to the client
async fn websocket_handler(
    ws: WebSocketUpgrade,
    AxumState(server_state): AxumState<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, server_state))
}

/// WebSocket connection handler
/// Subscribes to the event broadcaster and sends all events to the WebSocket client
async fn handle_websocket(socket: WebSocket, server_state: ServerState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to the event broadcaster
    let mut event_rx = server_state.observable_state.subscribe();

    println!("WebSocket client connected");

    // Send initial device state
    let state_json = {
        let state = server_state.observable_state.read();
        serde_json::to_string(&*state).ok()
    };

    if let Some(json) = state_json {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    // Spawn task to receive messages from client (ping/pong)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // Handle ping/pong and close messages
            if matches!(msg, Message::Close(_)) {
                break;
            }
        }
    });

    // Main task: forward events from broadcaster to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            // Serialize event to JSON
            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Failed to serialize event: {}", e);
                    continue;
                }
            };

            // Send to WebSocket client
            if sender.send(Message::Text(json.into())).await.is_err() {
                // Client disconnected
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        },
        _ = &mut recv_task => {
            send_task.abort();
        }
    }

    println!("WebSocket client disconnected");
}

/// Handler for GET /psets endpoint
/// Returns all available PSETs
async fn get_psets(
    AxumState(server_state): AxumState<ServerState>,
) -> impl IntoResponse {
    let repo = server_state.pset_repository.read().unwrap();
    let psets = repo.get_all();
    Json(psets)
}

/// Handler for GET /psets/:id endpoint
/// Returns a specific PSET by ID
async fn get_pset_by_id(
    AxumState(server_state): AxumState<ServerState>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    let repo = server_state.pset_repository.read().unwrap();
    match repo.get_by_id(id) {
        Some(pset) => (StatusCode::OK, Json(pset)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("PSET with id {} not found", id)
            })),
        )
            .into_response(),
    }
}

/// Handler for POST /psets/:id/select endpoint
/// Selects the specified PSET as the active parameter set
async fn select_pset(
    AxumState(server_state): AxumState<ServerState>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    // Check if PSET exists
    let pset_name = {
        let repo = server_state.pset_repository.read().unwrap();
        match repo.get_by_id(id) {
            Some(pset) => pset.name.clone(),
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("PSET with id {} not found", id)
                    })),
                )
                    .into_response()
            }
        }
    };

    // Set the PSET in device state and broadcast the change
    server_state
        .observable_state
        .set_pset(id, Some(pset_name.clone()));

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": format!("PSET {} '{}' selected", id, pset_name),
            "pset_id": id,
            "pset_name": pset_name
        })),
    )
        .into_response()
}

/// Handler for POST /psets endpoint
/// Creates a new PSET
async fn create_pset(
    AxumState(server_state): AxumState<ServerState>,
    Json(pset): Json<pset::Pset>,
) -> impl IntoResponse {
    let mut repo = server_state.pset_repository.write().unwrap();

    match repo.create(pset) {
        Ok(created_pset) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "message": "PSET created successfully",
                "pset": created_pset
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": err
            })),
        )
            .into_response(),
    }
}

/// Handler for PUT /psets/:id endpoint
/// Updates an existing PSET
async fn update_pset(
    AxumState(server_state): AxumState<ServerState>,
    Path(id): Path<u32>,
    Json(pset): Json<pset::Pset>,
) -> impl IntoResponse {
    let mut repo = server_state.pset_repository.write().unwrap();

    match repo.update(id, pset) {
        Ok(updated_pset) => {
            // If this is the currently selected PSET, update the state
            let current_pset_id = server_state.observable_state.read().current_pset_id;
            if current_pset_id == Some(id) {
                server_state
                    .observable_state
                    .set_pset(id, Some(updated_pset.name.clone()));
            }

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "PSET updated successfully",
                    "pset": updated_pset
                })),
            )
                .into_response()
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": err
            })),
        )
            .into_response(),
    }
}

/// Handler for DELETE /psets/:id endpoint
/// Deletes a PSET
async fn delete_pset(
    AxumState(server_state): AxumState<ServerState>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    // Check if this PSET is currently selected
    let current_pset_id = server_state.observable_state.read().current_pset_id;
    if current_pset_id == Some(id) {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "success": false,
                "error": "Cannot delete currently selected PSET. Please select another PSET first."
            })),
        )
            .into_response();
    }

    let mut repo = server_state.pset_repository.write().unwrap();

    match repo.delete(id) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "PSET deleted successfully"
            })),
        )
            .into_response(),
        Err(err) => {
            let status = if err.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };

            (
                status,
                Json(serde_json::json!({
                    "success": false,
                    "error": err
                })),
            )
                .into_response()
        }
    }
}
