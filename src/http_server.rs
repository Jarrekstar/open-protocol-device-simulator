use crate::config::Settings;
use crate::device_fsm::{DeviceFSM, DeviceFSMState, TighteningParams};
use crate::events::SimulatorEvent;
use crate::failure_simulator::FailureConfig;
use crate::handler::data::TighteningResult;
use crate::job::{Job, SharedJobRepository};
use crate::multi_spindle::{MultiSpindleStatus, generate_multi_spindle_results};
use crate::observable_state::ObservableState;
use crate::pset::{self, SharedPsetRepository};
use crate::state::DeviceState;
use axum::{
    Router,
    extract::{
        Path, State as AxumState, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};

/// Shared state for HTTP server
#[derive(Clone)]
pub struct ServerState {
    pub observable_state: ObservableState,
    pub auto_tightening_active: Arc<AtomicBool>,
    pub pset_repository: SharedPsetRepository,
    pub job_repository: SharedJobRepository,
    pub settings: Settings,
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
#[allow(clippy::too_many_arguments)]
fn build_tightening_result(
    state: &DeviceState,
    job_id: u32,
    pset_id: u32,
    batch_size: u32,
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
        job_id,
        pset_id,
        batch_size,
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

struct OperationContext {
    job_id: u32,
    pset_id: u32,
    batch_size: u32,
}

fn operation_context(state: &DeviceState) -> OperationContext {
    OperationContext {
        job_id: state.current_job_id.unwrap_or(1),
        pset_id: state.current_pset_id.unwrap_or(1),
        batch_size: state.tightening_tracker.batch_size(),
    }
}

#[allow(clippy::too_many_arguments)]
fn record_tightening_completion(
    observable_state: &ObservableState,
    pset_repository: &SharedPsetRepository,
    params: &TighteningParams,
    torque: f64,
    angle: f64,
    tightening_ok: bool,
    torque_ok: bool,
    angle_ok: bool,
    broadcast_tightening_result: bool,
) -> (u32, bool, u32) {
    let (
        result,
        batch_counter,
        batch_completed,
        target_size,
        job_progress,
        runtime,
        next_pset_id,
        tool_was_disabled,
    ) = {
        let mut state = observable_state.write();
        let context = operation_context(&state);
        let tool_was_enabled = state.tool_enabled;
        let info = state.add_tightening(tightening_ok);
        let result = build_tightening_result(
            &state,
            context.job_id,
            context.pset_id,
            context.batch_size,
            &info,
            torque,
            angle,
            tightening_ok,
            torque_ok,
            angle_ok,
            params,
        );
        let job_progress = info.job_progress.clone();
        let runtime = state.job_runtime_state();
        let next_pset_id = job_progress
            .as_ref()
            .filter(|progress| progress.step_changed)
            .and_then(|_| state.current_pset_id);
        (
            result,
            info.counter,
            state.tightening_tracker.is_complete(),
            state.tightening_tracker.batch_size(),
            job_progress,
            runtime,
            next_pset_id,
            tool_was_enabled && !state.tool_enabled,
        )
    };

    if broadcast_tightening_result {
        observable_state.broadcast(SimulatorEvent::TighteningCompleted { result });
    }

    if let (Some(progress), Some(runtime)) = (job_progress, runtime) {
        if let Some(pset_id) = next_pset_id {
            let pset_name = pset_repository
                .read()
                .unwrap()
                .get_by_id(pset_id)
                .map(|pset| pset.name)
                .unwrap_or_else(|| "Unknown".to_string());
            {
                let mut state = observable_state.write();
                state.current_pset_name = Some(pset_name.clone());
            }
            observable_state.broadcast(SimulatorEvent::PsetChanged { pset_id, pset_name });
            observable_state.broadcast(SimulatorEvent::JobStepChanged {
                state: runtime.clone(),
                previous_step: progress.previous_step_index as u32 + 1,
            });
        }
        observable_state.broadcast(SimulatorEvent::JobProgress {
            state: runtime.clone(),
        });
        if let Some(status) = progress.completed_status {
            let mut completed_state = runtime;
            completed_state.status = status;
            completed_state.total_progress = completed_state.total_batch_size;
            observable_state.broadcast(SimulatorEvent::JobCompleted {
                state: completed_state,
            });
        }
    } else if batch_completed {
        observable_state.broadcast(SimulatorEvent::BatchCompleted {
            total: batch_counter,
        });
    }

    if tool_was_disabled {
        observable_state.broadcast(SimulatorEvent::ToolStateChanged { enabled: false });
    }

    (batch_counter, batch_completed, target_size)
}

/// Create the HTTP router with all endpoints configured
pub fn create_router(observable_state: ObservableState, settings: Settings) -> Router {
    let db_path = settings.database.path.to_str().unwrap_or_else(|| {
        eprintln!(
            "Warning: Database path {:?} is not valid UTF-8, falling back to 'simulator.db'",
            settings.database.path
        );
        "simulator.db"
    });
    let pset_repository = crate::pset::create_sqlite_repository(db_path).unwrap_or_else(|e| {
        eprintln!(
            "Failed to create SQLite repository: {}. Falling back to in-memory.",
            e
        );
        crate::pset::create_default_repository()
    });
    let job_repository = crate::job::create_sqlite_repository(db_path).unwrap_or_else(|e| {
        eprintln!(
            "Failed to create SQLite Job repository: {}. Falling back to in-memory.",
            e
        );
        crate::job::create_default_repository()
    });

    create_router_with_repositories(observable_state, settings, pset_repository, job_repository)
}

pub fn create_router_with_repositories(
    observable_state: ObservableState,
    settings: Settings,
    pset_repository: SharedPsetRepository,
    job_repository: SharedJobRepository,
) -> Router {
    let server_state = ServerState {
        observable_state,
        auto_tightening_active: Arc::new(AtomicBool::new(false)),
        pset_repository,
        job_repository,
        settings,
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
        .route(
            "/config/failure",
            get(get_failure_config).post(update_failure_config),
        )
        .route("/psets", get(get_psets).post(create_pset))
        .route(
            "/psets/{id}",
            get(get_pset_by_id).put(update_pset).delete(delete_pset),
        )
        .route("/psets/{id}/select", post(select_pset))
        .route("/jobs", get(get_jobs).post(create_job))
        .route("/jobs/active/clear", post(clear_active_job))
        .route(
            "/jobs/{id}",
            get(get_job_by_id).put(update_job).delete(delete_job),
        )
        .route("/jobs/{id}/select", post(select_job))
        .route("/jobs/{id}/restart", post(restart_job))
        .route("/ws/events", get(websocket_handler))
        .layer(cors)
        .with_state(server_state)
}

/// Start the HTTP server for state inspection and simulation control
pub async fn start_http_server(observable_state: ObservableState, settings: Settings) {
    let db_path = settings.database.path.to_str().unwrap_or("simulator.db");
    let psets = crate::pset::create_sqlite_repository(db_path)
        .unwrap_or_else(|_| crate::pset::create_default_repository());
    let jobs = crate::job::create_sqlite_repository(db_path)
        .unwrap_or_else(|_| crate::job::create_default_repository());
    start_http_server_with_repositories(observable_state, settings, psets, jobs).await;
}

pub async fn start_http_server_with_repositories(
    observable_state: ObservableState,
    settings: Settings,
    pset_repository: SharedPsetRepository,
    job_repository: SharedJobRepository,
) {
    let bind_addr = format!(
        "{}:{}",
        settings.server.bind_address, settings.server.http_port
    );
    let app = create_router_with_repositories(
        observable_state,
        settings,
        pset_repository,
        job_repository,
    );

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind HTTP server to {}", bind_addr));

    println!("HTTP state server listening on http://{}", bind_addr);
    println!("Endpoints:");
    println!("  GET    /state                     - View device state");
    println!("  POST   /simulate/tightening       - Simulate a single tightening operation");
    println!(
        "  POST   /auto-tightening/start     - Start automated tightening simulation (continuous)"
    );
    println!("  POST   /auto-tightening/stop      - Stop automated tightening simulation");
    println!("  GET    /auto-tightening/status    - Get auto-tightening status");
    println!("  POST   /config/multi-spindle      - Configure multi-spindle mode");
    println!("  GET    /config/failure            - Get failure injection configuration");
    println!("  POST   /config/failure            - Update failure injection configuration");
    println!("  GET    /psets                     - Get all PSETs");
    println!("  POST   /psets                     - Create a new PSET");
    println!("  GET    /psets/{{id}}                - Get a specific PSET by ID");
    println!("  PUT    /psets/{{id}}                - Update a PSET");
    println!("  DELETE /psets/{{id}}                - Delete a PSET");
    println!("  POST   /psets/{{id}}/select         - Select a PSET as active");
    println!("  GET    /jobs                     - Get all Jobs");
    println!("  POST   /jobs                     - Create a Job");
    println!("  GET    /jobs/{{id}}                - Get a Job");
    println!("  PUT    /jobs/{{id}}                - Update a Job");
    println!("  DELETE /jobs/{{id}}                - Delete a Job");
    println!("  POST   /jobs/{{id}}/select         - Select a Job");
    println!("  POST   /jobs/{{id}}/restart        - Restart the active Job");
    println!("  POST   /jobs/active/clear        - Exit completed JobMode");
    println!("  GET    /ws/events                 - WebSocket event stream");

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
    /// Optional torque override (if provided, used as exact target with min=max)
    torque: Option<f64>,
    /// Optional angle override (if provided, used as exact target with min=max)
    angle: Option<f64>,
    /// Optional OK/NOK override (None = FSM decides, Some(true) = Force OK, Some(false) = Force NOK)
    ok: Option<bool>,
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
    let (tool_enabled, job_mode, job_running) = {
        let state = server_state.observable_state.read();
        (
            state.tool_enabled,
            state.is_job_mode(),
            state.is_job_running(),
        )
    };

    if !tool_enabled {
        return (
            StatusCode::CONFLICT,
            Json(TighteningResponse {
                success: false,
                message: "Cannot simulate tightening: tool is disabled".to_string(),
                batch_counter: 0,
                subscribers: 0,
            }),
        );
    }
    if job_mode && !job_running {
        return (
            StatusCode::CONFLICT,
            Json(TighteningResponse {
                success: false,
                message: "The active Job is complete. Restart it or exit JobMode.".to_string(),
                batch_counter: 0,
                subscribers: 0,
            }),
        );
    }
    if job_running && (payload.torque.is_some() || payload.angle.is_some()) {
        return (
            StatusCode::CONFLICT,
            Json(TighteningResponse {
                success: false,
                message: "Manual torque and angle overrides are disabled while a Job is running"
                    .to_string(),
                batch_counter: 0,
                subscribers: 0,
            }),
        );
    }

    // Determine tightening params: use overrides if provided, otherwise use PSET
    let params = match (payload.torque, payload.angle) {
        (Some(torque), Some(angle)) => {
            // Manual override: use exact values (min=max)
            println!(
                "Manual tightening override: Torque={:.1} Nm, Angle={:.1}°",
                torque, angle
            );
            TighteningParams {
                target_torque: torque,
                torque_min: torque,
                torque_max: torque,
                target_angle: angle,
                angle_min: angle,
                angle_max: angle,
                duration_ms: 500,
            }
        }
        _ => {
            // Use PSET values
            let state = server_state.observable_state.read();
            get_tightening_params(
                state.current_pset_id,
                &server_state.pset_repository,
                500, // duration_ms for simulation
            )
        }
    };

    println!(
        "Simulating tightening with params: Torque {:.1}-{:.1} Nm (target: {:.1}), Angle {:.1}-{:.1}° (target: {:.1})",
        params.torque_min,
        params.torque_max,
        params.target_torque,
        params.angle_min,
        params.angle_max,
        params.target_angle
    );

    // Run FSM simulation
    let fsm = DeviceFSM::new();
    let fsm = fsm.start_tightening(params.clone());
    tokio::time::sleep(Duration::from_millis(10)).await; // Brief simulation
    let fsm = fsm.complete();
    let fsm_outcome = fsm.result();

    // Apply manual OK/NOK override if provided, otherwise use FSM result
    let final_ok = if let Some(force_ok) = payload.ok {
        println!(
            "Forcing result: {} (FSM determined: {})",
            if force_ok { "OK" } else { "NOK" },
            if fsm_outcome.ok { "OK" } else { "NOK" }
        );
        force_ok
    } else {
        fsm_outcome.ok
    };

    println!(
        "Result: Torque={:.2} Nm ({}), Angle={:.1}° ({}), Overall: {}",
        fsm_outcome.actual_torque,
        if fsm_outcome.torque_ok { "OK" } else { "NOK" },
        fsm_outcome.actual_angle,
        if fsm_outcome.angle_ok { "OK" } else { "NOK" },
        if final_ok { "OK" } else { "NOK" }
    );

    let (batch_counter, batch_completed, _) = record_tightening_completion(
        &server_state.observable_state,
        &server_state.pset_repository,
        &params,
        fsm_outcome.actual_torque,
        fsm_outcome.actual_angle,
        final_ok,
        fsm_outcome.torque_ok,
        fsm_outcome.angle_ok,
        true,
    );
    if batch_completed && !job_mode {
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
    /// Time between tightening cycles in milliseconds (uses config default if not specified)
    interval_ms: Option<u64>,
    /// Duration of each tightening operation in milliseconds (uses config default if not specified)
    duration_ms: Option<u64>,
    /// Probability of failure (0.0 = never fail, 1.0 = always fail, uses config default if not specified)
    failure_rate: Option<f64>,
}

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

    // Use request values or fall back to configuration defaults
    let defaults = &server_state.settings.defaults;
    let interval_ms = payload
        .interval_ms
        .unwrap_or(defaults.auto_tightening_interval_ms);
    let duration_ms = payload
        .duration_ms
        .unwrap_or(defaults.auto_tightening_duration_ms);
    let failure_rate = payload
        .failure_rate
        .unwrap_or(defaults.failure_rate)
        .clamp(0.0, 1.0);

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
            if let Some(0) = remaining {
                tokio::time::sleep(Duration::from_millis(interval_ms)).await;
                continue;
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
                let (result_id, job_id, pset_id, operation_batch_size) = {
                    let s = observable_state.read();
                    (
                        s.tightening_tracker.tightening_sequence() + 1, // Next sequence number
                        s.current_job_id.unwrap_or(1),
                        s.current_pset_id.unwrap_or(1),
                        s.tightening_tracker.batch_size(),
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

                // Broadcast "Completed" status (MID 0091)
                let completed_status = MultiSpindleStatus::completed(
                    multi_spindle_config.sync_id,
                    multi_spindle_config.spindle_count,
                );
                observable_state.broadcast(SimulatorEvent::MultiSpindleStatusCompleted {
                    status: completed_status,
                });

                let (batch_counter, batch_completed, target_size) = record_tightening_completion(
                    &observable_state,
                    &pset_repository,
                    &params,
                    outcome.actual_torque,
                    outcome.actual_angle,
                    overall_ok,
                    overall_ok,
                    overall_ok,
                    false,
                );

                let batch_status = {
                    let state = observable_state.read();
                    state
                        .current_job_status
                        .map(|status| status.protocol_value())
                        .unwrap_or(if batch_completed { 1 } else { 2 })
                };
                observable_state.broadcast(SimulatorEvent::MultiSpindleResultCompleted {
                    result: multi_result,
                    job_id,
                    pset_id,
                    batch_size: operation_batch_size,
                    batch_counter,
                    batch_status,
                });

                // Broadcast auto-tightening progress
                let is_running = auto_active.load(Ordering::Relaxed);
                observable_state.broadcast_auto_progress(batch_counter, target_size, is_running);

                if batch_completed && !observable_state.read().is_job_mode() {
                    println!("Batch completed with {} tightenings", batch_counter);
                }
            } else {
                // ============================================================
                // SINGLE-SPINDLE PATH
                // ============================================================

                let (batch_counter, batch_completed, target_size) = record_tightening_completion(
                    &observable_state,
                    &pset_repository,
                    &params,
                    outcome.actual_torque,
                    outcome.actual_angle,
                    final_ok,
                    outcome.torque_ok,
                    outcome.angle_ok,
                    true,
                );

                // Broadcast auto-tightening progress
                let is_running = auto_active.load(Ordering::Relaxed);
                observable_state.broadcast_auto_progress(batch_counter, target_size, is_running);

                if batch_completed && !observable_state.read().is_job_mode() {
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

        // Broadcast stopped status so frontend updates
        let (counter, target_size) = {
            let state = observable_state.read();
            (
                state.tightening_tracker.counter(),
                state.tightening_tracker.batch_size(),
            )
        };
        observable_state.broadcast_auto_progress(counter, target_size, false);

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

        server_state
            .observable_state
            .broadcast_auto_progress(counter, target_size, false);

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
// Failure Injection Configuration
// ============================================================================

#[derive(Deserialize)]
struct FailureConfigRequest {
    /// Optional: set connection health directly (0-100)
    /// If provided, this recalculates all other failure rates
    connection_health: Option<u8>,

    /// Optional: full manual configuration
    /// If connection_health is not provided, these values are used directly
    enabled: Option<bool>,
    packet_loss_rate: Option<f64>,
    delay_min_ms: Option<u64>,
    delay_max_ms: Option<u64>,
    corruption_rate: Option<f64>,
    force_disconnect_rate: Option<f64>,
}

/// Handler for GET /config/failure endpoint
/// Returns the current failure injection configuration
async fn get_failure_config(
    AxumState(server_state): AxumState<ServerState>,
) -> Json<FailureConfig> {
    let state = server_state.observable_state.read();
    Json(state.failure_config.clone())
}

/// Handler for POST /config/failure endpoint
/// Updates the failure injection configuration
async fn update_failure_config(
    AxumState(server_state): AxumState<ServerState>,
    Json(payload): Json<FailureConfigRequest>,
) -> impl IntoResponse {
    let new_config = if let Some(health) = payload.connection_health {
        // Simple mode: use connection health slider
        let health_clamped = health.min(100);
        println!(
            "Updating failure config via connection health: {}%",
            health_clamped
        );
        FailureConfig::from_health(health_clamped)
    } else {
        // Advanced mode: update individual fields
        let mut config = {
            let state = server_state.observable_state.read();
            state.failure_config.clone()
        };

        if let Some(enabled) = payload.enabled {
            config.enabled = enabled;
        }
        if let Some(rate) = payload.packet_loss_rate {
            config.packet_loss_rate = rate.clamp(0.0, 1.0);
        }
        if let Some(min) = payload.delay_min_ms {
            config.delay_min_ms = min;
        }
        if let Some(max) = payload.delay_max_ms {
            config.delay_max_ms = max;
        }
        if let Some(rate) = payload.corruption_rate {
            config.corruption_rate = rate.clamp(0.0, 1.0);
        }
        if let Some(rate) = payload.force_disconnect_rate {
            config.force_disconnect_rate = rate.clamp(0.0, 1.0);
        }

        println!("Updating failure config via individual fields");
        config
    };

    // Validate the configuration
    if !new_config.is_valid() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "Invalid failure configuration: check that all values are within valid ranges"
            })),
        )
            .into_response();
    }

    // Update the state
    {
        let mut state = server_state.observable_state.write();
        state.failure_config = new_config.clone();
    }

    println!("Failure injection config updated:");
    println!("  Enabled: {}", new_config.enabled);
    println!("  Connection Health: {}%", new_config.connection_health);
    println!("  Packet Loss: {:.1}%", new_config.packet_loss_rate * 100.0);
    println!(
        "  Delay: {}-{} ms",
        new_config.delay_min_ms, new_config.delay_max_ms
    );
    println!("  Corruption: {:.1}%", new_config.corruption_rate * 100.0);
    println!(
        "  Disconnect: {:.1}%",
        new_config.force_disconnect_rate * 100.0
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Failure injection configuration updated",
            "config": new_config
        })),
    )
        .into_response()
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

    // Clone sender for recv_task (need to share between tasks)
    let (pong_tx, mut pong_rx) = tokio::sync::mpsc::channel::<String>(10);

    // Spawn task to receive messages from client (handle ping/pong)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Try to parse as JSON to check if it's a ping message
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text)
                        && value.get("type").and_then(|t| t.as_str()) == Some("ping")
                    {
                        // Send pong response
                        let pong_msg = r#"{"type":"pong"}"#.to_string();
                        let _ = pong_tx.send(pong_msg).await;
                    }
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    });

    // Main task: forward events from broadcaster to WebSocket and handle pong responses
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Handle incoming events from broadcaster
                result = event_rx.recv() => {
                    match result {
                        Ok(event) => {
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
                        Err(_) => {
                            // Channel closed
                            break;
                        }
                    }
                }
                // Handle pong responses from recv_task
                Some(pong_msg) = pong_rx.recv() => {
                    if sender.send(Message::Text(pong_msg.into())).await.is_err() {
                        // Client disconnected
                        break;
                    }
                }
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
async fn get_psets(AxumState(server_state): AxumState<ServerState>) -> impl IntoResponse {
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
    if server_state.observable_state.read().is_job_mode() {
        return api_error(
            StatusCode::CONFLICT,
            "PSET selection is controlled by the active Job. Exit JobMode first.",
        );
    }
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
                    .into_response();
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
    if server_state
        .job_repository
        .read()
        .unwrap()
        .references_pset(id)
    {
        return api_error(
            StatusCode::CONFLICT,
            format!("Cannot delete PSET {id}: it is referenced by a Job"),
        );
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

fn api_error(status: StatusCode, message: impl Into<String>) -> axum::response::Response {
    let message = message.into();
    (
        status,
        Json(serde_json::json!({
            "success": false,
            "message": message,
            "error": message
        })),
    )
        .into_response()
}

fn validate_job(server_state: &ServerState, job: &Job) -> Result<(), (StatusCode, String)> {
    let pset_ids = {
        let repository = server_state.pset_repository.read().unwrap();
        let all = repository.get_all();
        for step in &job.steps {
            if repository.get_by_id(step.pset_id).is_none() {
                return Err((
                    StatusCode::NOT_FOUND,
                    format!("PSET {} referenced by the Job does not exist", step.pset_id),
                ));
            }
        }
        all.into_iter().map(|pset| pset.id).collect::<Vec<_>>()
    };
    let channel_id = server_state.observable_state.read().channel_id;
    job.validate(&[channel_id], &pset_ids)
        .map_err(|message| (StatusCode::BAD_REQUEST, message))
}

async fn get_jobs(AxumState(server_state): AxumState<ServerState>) -> impl IntoResponse {
    let jobs = server_state.job_repository.read().unwrap().get_all();
    Json(jobs)
}

async fn get_job_by_id(
    AxumState(server_state): AxumState<ServerState>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    match server_state.job_repository.read().unwrap().get_by_id(id) {
        Some(job) => (StatusCode::OK, Json(job)).into_response(),
        None => api_error(StatusCode::NOT_FOUND, format!("Job {id:02} not found")),
    }
}

async fn create_job(
    AxumState(server_state): AxumState<ServerState>,
    Json(job): Json<Job>,
) -> impl IntoResponse {
    if let Err((status, message)) = validate_job(&server_state, &job) {
        return api_error(status, message);
    }
    match server_state.job_repository.write().unwrap().create(job) {
        Ok(job) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "message": "Job created successfully",
                "job": job
            })),
        )
            .into_response(),
        Err(message) => api_error(StatusCode::BAD_REQUEST, message),
    }
}

async fn update_job(
    AxumState(server_state): AxumState<ServerState>,
    Path(id): Path<u32>,
    Json(mut job): Json<Job>,
) -> impl IntoResponse {
    if server_state.observable_state.read().is_job_running()
        && server_state.observable_state.read().current_job_id == Some(id)
    {
        return api_error(StatusCode::CONFLICT, "Cannot update a running Job");
    }
    if server_state
        .job_repository
        .read()
        .unwrap()
        .get_by_id(id)
        .is_none()
    {
        return api_error(StatusCode::NOT_FOUND, format!("Job {id:02} not found"));
    }
    job.id = id;
    if let Err((status, message)) = validate_job(&server_state, &job) {
        return api_error(status, message);
    }
    match server_state.job_repository.write().unwrap().update(id, job) {
        Ok(job) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Job updated successfully",
                "job": job
            })),
        )
            .into_response(),
        Err(message) => api_error(StatusCode::BAD_REQUEST, message),
    }
}

async fn delete_job(
    AxumState(server_state): AxumState<ServerState>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    if server_state.observable_state.read().is_job_running()
        && server_state.observable_state.read().current_job_id == Some(id)
    {
        return api_error(StatusCode::CONFLICT, "Cannot delete a running Job");
    }
    match server_state.job_repository.write().unwrap().delete(id) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Job deleted successfully"
            })),
        )
            .into_response(),
        Err(message) if message.contains("not found") => api_error(StatusCode::NOT_FOUND, message),
        Err(message) => api_error(StatusCode::BAD_REQUEST, message),
    }
}

async fn select_job(
    AxumState(server_state): AxumState<ServerState>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    if server_state.observable_state.read().is_job_running() {
        return api_error(StatusCode::CONFLICT, "A Job is already running");
    }
    let Some(job) = server_state.job_repository.read().unwrap().get_by_id(id) else {
        return api_error(StatusCode::NOT_FOUND, format!("Job {id:02} not found"));
    };
    let pset_name = server_state
        .pset_repository
        .read()
        .unwrap()
        .get_by_id(job.steps[0].pset_id)
        .map(|pset| pset.name);
    match server_state.observable_state.select_job(job, pset_name) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("Job {id:02} selected")
            })),
        )
            .into_response(),
        Err(message) => api_error(StatusCode::CONFLICT, message),
    }
}

async fn restart_job(
    AxumState(server_state): AxumState<ServerState>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    if server_state
        .job_repository
        .read()
        .unwrap()
        .get_by_id(id)
        .is_none()
    {
        return api_error(StatusCode::NOT_FOUND, format!("Job {id:02} not found"));
    }
    let first_pset_id = {
        let state = server_state.observable_state.read();
        let Some(execution) = state.tightening_tracker.job_execution() else {
            return api_error(StatusCode::CONFLICT, "Job is not active");
        };
        if execution.job.id != id {
            return api_error(StatusCode::CONFLICT, "Requested Job is not active");
        }
        execution.job.steps[0].pset_id
    };
    let pset_name = server_state
        .pset_repository
        .read()
        .unwrap()
        .get_by_id(first_pset_id)
        .map(|pset| pset.name);
    match server_state.observable_state.restart_job(id, pset_name) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("Job {id:02} restarted")
            })),
        )
            .into_response(),
        Err(message) => api_error(StatusCode::CONFLICT, message),
    }
}

async fn clear_active_job(AxumState(server_state): AxumState<ServerState>) -> impl IntoResponse {
    match server_state.observable_state.clear_job_mode() {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "JobMode cleared"
            })),
        )
            .into_response(),
        Err(message) => api_error(StatusCode::CONFLICT, message),
    }
}
