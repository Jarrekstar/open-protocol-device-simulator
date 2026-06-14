mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use std::sync::{Arc, RwLock};
use tower::ServiceExt;

/// Test GET /state endpoint
#[tokio::test]
async fn test_get_state_endpoint() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server,
    };

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);

    let app = http_server::create_router(observable_state, config::Settings::default());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/state")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse response body
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let state_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify state fields
    assert_eq!(state_json["cell_id"], 1);
    assert_eq!(state_json["channel_id"], 1);
    assert_eq!(state_json["controller_name"], "OpenProtocolSimulator");
    assert_eq!(state_json["tool_enabled"], true);
    assert_eq!(state_json["operation_mode"], "pset");
    assert_eq!(state_json["batch_size"], 0);
    assert_eq!(state_json["batch_counter"], 0);
}

/// Test POST /simulate/tightening endpoint
#[tokio::test]
async fn test_simulate_tightening_endpoint() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server,
    };

    let state = Arc::new(RwLock::new(DeviceState::new()));
    {
        let mut s = state.write().unwrap();
        s.set_batch_size(5); // Enable batch mode for counter tracking
    }

    let (broadcaster, _receiver) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let app = http_server::create_router(observable_state, config::Settings::default());

    let payload = json!({
        "torque": 12.5,
        "angle": 40.0,
        "ok": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/simulate/tightening")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse response
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["success"], true);
    assert_eq!(result["batch_counter"], 1);
}

/// Test POST /simulate/tightening endpoint when tool is disabled
#[tokio::test]
async fn test_simulate_tightening_endpoint_rejects_when_tool_disabled() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server,
    };

    let state = Arc::new(RwLock::new(DeviceState::new()));
    {
        let mut s = state.write().unwrap();
        s.set_batch_size(5);
        s.disable_tool();
    }

    let (broadcaster, mut receiver) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let _keepalive_sender = broadcaster.clone();
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let app = http_server::create_router(observable_state, config::Settings::default());

    let payload = json!({
        "torque": 12.5,
        "angle": 40.0,
        "ok": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/simulate/tightening")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["success"], false);
    assert!(
        result["message"]
            .as_str()
            .unwrap()
            .contains("tool is disabled")
    );

    let s = state.read().unwrap();
    assert_eq!(s.tightening_tracker.counter(), 0);

    assert!(matches!(
        receiver.try_recv(),
        Err(tokio::sync::broadcast::error::TryRecvError::Empty)
    ));
}

/// Test POST /auto-tightening/start endpoint
#[tokio::test]
async fn test_start_auto_tightening_endpoint() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server,
    };

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let app = http_server::create_router(observable_state, config::Settings::default());

    let payload = json!({
        "interval_ms": 1000,
        "duration_ms": 100,
        "failure_rate": 0.0
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/auto-tightening/start")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse response
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["success"], true);
    assert!(result["message"].as_str().unwrap().contains("started"));
}

/// Test POST /auto-tightening/start conflict (already running)
#[tokio::test]
async fn test_start_auto_tightening_conflict() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server,
    };
    use std::sync::atomic::AtomicBool;

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);

    // Create server state with auto-tightening already active
    let pset_repository = open_protocol_device_simulator::pset::create_default_repository();
    let job_repository = open_protocol_device_simulator::job::create_default_repository();
    let server_state = http_server::ServerState {
        observable_state,
        auto_tightening_active: Arc::new(AtomicBool::new(true)), // Already running
        pset_repository,
        job_repository,
        settings: config::Settings::default(),
        protocol_configuration: open_protocol_device_simulator::ProtocolConfiguration::default(),
    };

    let app = axum::Router::new()
        .route(
            "/auto-tightening/start",
            axum::routing::post(
                |_state: axum::extract::State<http_server::ServerState>,
                 _payload: axum::Json<serde_json::Value>| async {
                    (
                        StatusCode::CONFLICT,
                        axum::Json(json!({"success": false, "message": "Already running"})),
                    )
                },
            ),
        )
        .with_state(server_state);

    let payload = json!({
        "interval_ms": 1000,
        "duration_ms": 100
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/auto-tightening/start")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

/// Test POST /auto-tightening/stop endpoint
#[tokio::test]
async fn test_stop_auto_tightening_endpoint() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server,
    };

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let app = http_server::create_router(observable_state, config::Settings::default());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/auto-tightening/stop")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse response
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["success"], true);
}

/// Test GET /auto-tightening/status endpoint
#[tokio::test]
async fn test_get_auto_tightening_status_endpoint() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server,
    };

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let app = http_server::create_router(observable_state, config::Settings::default());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/auto-tightening/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse response
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["running"], false);
    assert!(result["counter"].is_number());
    assert!(result["target_size"].is_number());
}

#[tokio::test]
async fn test_protocol_catalog_and_profile_endpoints() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, ProtocolConfiguration, SimulatorEvent, config, http_server,
        job, pset,
    };

    let state = DeviceState::new_shared();
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let protocol_configuration = ProtocolConfiguration::default();
    let app = http_server::create_router_with_repositories_and_protocol(
        ObservableState::new(state, broadcaster),
        config::Settings::default(),
        pset::create_default_repository(),
        job::create_default_repository(),
        protocol_configuration.clone(),
    );

    let catalog_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/protocol/catalog")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(catalog_response.status(), StatusCode::OK);
    let body = catalog_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let catalog: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let tightening = catalog
        .as_array()
        .unwrap()
        .iter()
        .find(|family| family["id"] == "tightening_result")
        .unwrap();
    assert!(
        tightening["supported_revisions"]
            .as_array()
            .unwrap()
            .contains(&json!(998))
    );

    let mut profile = protocol_configuration.profile();
    profile.families.get_mut("communication").unwrap().revision = 6;
    profile.samples.station_id = 12;
    let update_response = app
        .oneshot(
            Request::builder()
                .uri("/protocol/profile")
                .method("PUT")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&profile).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);
    assert_eq!(
        protocol_configuration
            .profile()
            .families
            .get("communication")
            .unwrap()
            .revision,
        6
    );
    assert_eq!(protocol_configuration.samples().station_id, 12);
}

fn job_payload(id: u32, pset_id: u32) -> serde_json::Value {
    json!({
        "id": id,
        "name": "Wheel Job",
        "forced_order": 1,
        "first_tightening_timeout": 0,
        "job_timeout": 0,
        "batch_count_mode": 0,
        "lock_at_job_done": false,
        "use_line_control": false,
        "repeat_job": false,
        "loosening_mode": 0,
        "repair_mode": 0,
        "steps": [{
            "channel_id": 1,
            "pset_id": pset_id,
            "auto_value": true,
            "batch_size": 1
        }]
    })
}

#[tokio::test]
async fn test_operation_mode_endpoint_selects_command_profiles() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, OperationMode, SimulatorEvent, config, handler, http_server,
        job, protocol, pset,
    };

    let state = DeviceState::new_shared();
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let registry = handler::create_default_registry(observable_state.clone());
    let app = http_server::create_router_with_repositories(
        observable_state,
        config::Settings::default(),
        pset::create_default_repository(),
        job::create_default_repository(),
    );

    let select_batch_mode = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/config/operation-mode")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"mode":"batch"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(select_batch_mode.status(), StatusCode::OK);
    {
        let current = state.read().unwrap();
        assert_eq!(current.operation_mode(), OperationMode::Batch);
        assert_eq!(current.tightening_tracker.batch_size(), 0);
        assert!(!current.tightening_tracker.should_wait_for_config());
        assert_eq!(current.current_job_id, None);
    }

    let accepted_pset_in_batch = registry
        .handle_message(&protocol::Message {
            length: 23,
            mid: 18,
            revision: 1,
            data: b"002".to_vec(),
        })
        .unwrap();
    assert_eq!(accepted_pset_in_batch.mid, 16);
    assert_eq!(state.read().unwrap().current_pset_id, Some(2));

    // Job uploads and subscriptions are never profile-gated (spec §3.7.3)
    let job_upload_in_batch = registry
        .handle_message(&protocol::Message {
            length: 20,
            mid: 30,
            revision: 1,
            data: Vec::new(),
        })
        .unwrap();
    assert_eq!(job_upload_in_batch.mid, 31);

    // Job selection outside the Job profile: error 20 "Job can not be set"
    let rejected_job_select = registry
        .handle_message(&protocol::Message {
            length: 22,
            mid: 38,
            revision: 1,
            data: b"01".to_vec(),
        })
        .unwrap();
    assert_eq!(rejected_job_select.mid, 4);
    assert_eq!(rejected_job_select.data, b"003820");

    let response = registry
        .handle_message(&protocol::Message {
            length: 25,
            mid: 19,
            revision: 1,
            data: b"00220".to_vec(),
        })
        .unwrap();
    assert_eq!(response.mid, 5);
    assert_eq!(state.read().unwrap().tightening_tracker.batch_size(), 20);

    let select_pset_mode = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/config/operation-mode")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"mode":"pset"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(select_pset_mode.status(), StatusCode::OK);
    assert_eq!(state.read().unwrap().operation_mode(), OperationMode::Pset);

    let accepted_pset = registry
        .handle_message(&protocol::Message {
            length: 23,
            mid: 18,
            revision: 1,
            data: b"002".to_vec(),
        })
        .unwrap();
    assert_eq!(accepted_pset.mid, 16);

    // Batch size is a runtime Pset attribute (spec MID 0019), accepted in the
    // Pset profile; accepting it switches the device to batch counting.
    let accepted_batch_size = registry
        .handle_message(&protocol::Message {
            length: 25,
            mid: 19,
            revision: 1,
            data: b"00220".to_vec(),
        })
        .unwrap();
    assert_eq!(accepted_batch_size.mid, 5);
    assert_eq!(state.read().unwrap().operation_mode(), OperationMode::Batch);

    // Job restart is not profile-gated; without a running Job it answers
    // error 21 "Job not running".
    let rejected_restart = registry
        .handle_message(&protocol::Message {
            length: 22,
            mid: 39,
            revision: 1,
            data: b"01".to_vec(),
        })
        .unwrap();
    assert_eq!(rejected_restart.mid, 4);
    assert_eq!(rejected_restart.data, b"003921");

    let select_job_mode = app
        .oneshot(
            Request::builder()
                .uri("/config/operation-mode")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"mode":"job"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(select_job_mode.status(), StatusCode::OK);
    assert_eq!(state.read().unwrap().operation_mode(), OperationMode::Job);

    let accepted_job = registry
        .handle_message(&protocol::Message {
            length: 20,
            mid: 30,
            revision: 1,
            data: Vec::new(),
        })
        .unwrap();
    assert_eq!(accepted_job.mid, 31);

    // In the Job profile: Pset selection answers error 03 "Parameter set can
    // not be set"; batch configuration answers error 01 "Invalid data".
    let rejections: [(u16, &[u8], &[u8]); 3] = [
        (18, b"002", b"001803"),
        (19, b"00220", b"001901"),
        (20, b"002", b"002001"),
    ];
    for (mid, data, expected) in rejections {
        let rejected = registry
            .handle_message(&protocol::Message {
                length: (20 + data.len()) as u32,
                mid,
                revision: 1,
                data: data.to_vec(),
            })
            .unwrap();
        assert_eq!(rejected.mid, 4);
        assert_eq!(rejected.data, expected);
    }
}

#[tokio::test]
async fn test_job_crud_and_runtime_endpoints() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server, job, pset,
    };

    let state = DeviceState::new_shared();
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable = ObservableState::new(Arc::clone(&state), broadcaster);
    let app = http_server::create_router_with_repositories(
        observable,
        config::Settings::default(),
        pset::create_default_repository(),
        job::create_default_repository(),
    );

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/jobs")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&job_payload(7, 2)).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);

    let referenced_pset_delete = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/psets/2")
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(referenced_pset_delete.status(), StatusCode::CONFLICT);

    let list = app
        .clone()
        .oneshot(Request::builder().uri("/jobs").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = list.into_body().collect().await.unwrap().to_bytes();
    let jobs: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(jobs.as_array().unwrap().len(), 1);

    let rejected_select = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/jobs/7/select")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected_select.status(), StatusCode::CONFLICT);

    let select_job_mode = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/config/operation-mode")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"mode":"job"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(select_job_mode.status(), StatusCode::OK);

    let select = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/jobs/7/select")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(select.status(), StatusCode::OK);
    assert!(state.read().unwrap().is_job_running());

    let missing_select = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/jobs/99/select")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_select.status(), StatusCode::NOT_FOUND);
    assert_eq!(state.read().unwrap().current_job_id, Some(7));

    let update_conflict = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/jobs/7")
                .method("PUT")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&job_payload(7, 2)).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_conflict.status(), StatusCode::CONFLICT);

    let tightening = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/simulate/tightening")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"ok":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tightening.status(), StatusCode::OK);
    assert!(!state.read().unwrap().is_job_running());
    assert!(!state.read().unwrap().is_job_mode());
    assert_eq!(
        state.read().unwrap().operation_mode(),
        open_protocol_device_simulator::OperationMode::Job
    );
    assert_eq!(state.read().unwrap().current_job_id, None);
    assert_eq!(state.read().unwrap().current_job_status, None);

    let clear = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/jobs/active/clear")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(clear.status(), StatusCode::OK);

    let delete = app
        .oneshot(
            Request::builder()
                .uri("/jobs/7")
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_repeating_job_stays_active_and_reports_restart_state() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server, job, pset,
    };

    let state = DeviceState::new_shared();
    let (broadcaster, mut receiver) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let app = http_server::create_router_with_repositories(
        ObservableState::new(Arc::clone(&state), broadcaster),
        config::Settings::default(),
        pset::create_default_repository(),
        job::create_default_repository(),
    );
    let mut payload = job_payload(8, 1);
    payload["repeat_job"] = json!(true);

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/jobs")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);

    state.write().unwrap().set_job_mode();
    let select = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/jobs/8/select")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(select.status(), StatusCode::OK);

    let tightening = app
        .oneshot(
            Request::builder()
                .uri("/simulate/tightening")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"ok":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tightening.status(), StatusCode::OK);

    let current = state.read().unwrap();
    assert!(current.is_job_running());
    assert_eq!(current.current_job_id, Some(8));
    assert_eq!(
        current.current_job_status,
        Some(open_protocol_device_simulator::job::JobStatus::Running)
    );
    assert_eq!(current.current_job_total_progress, 0);
    drop(current);

    let events = std::iter::from_fn(|| receiver.try_recv().ok()).collect::<Vec<_>>();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SimulatorEvent::JobCompleted { repeated: true, .. }))
    );
    assert!(events.iter().any(|event| matches!(
        event,
        SimulatorEvent::JobProgress { state }
            if state.status == open_protocol_device_simulator::job::JobStatus::Running
                && state.total_progress == 0
    )));
}

#[tokio::test]
async fn test_auto_tightening_stops_when_job_finishes() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server, job, pset,
    };

    let state = DeviceState::new_shared();
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let app = http_server::create_router_with_repositories(
        ObservableState::new(Arc::clone(&state), broadcaster),
        config::Settings::default(),
        pset::create_default_repository(),
        job::create_default_repository(),
    );

    app.clone()
        .oneshot(
            Request::builder()
                .uri("/jobs")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&job_payload(9, 1)).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    state.write().unwrap().set_job_mode();
    app.clone()
        .oneshot(
            Request::builder()
                .uri("/jobs/9/select")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let start = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auto-tightening/start")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"interval_ms":1,"duration_ms":1,"failure_rate":0.0}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(start.status(), StatusCode::OK);

    for _ in 0..50 {
        if !state.read().unwrap().is_job_mode() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;
    }
    assert!(!state.read().unwrap().is_job_mode());
    assert_eq!(state.read().unwrap().current_job_id, None);

    let status = app
        .oneshot(
            Request::builder()
                .uri("/auto-tightening/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = status.into_body().collect().await.unwrap().to_bytes();
    let status: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(status["running"], false);
}

#[tokio::test]
async fn test_job_validation_and_missing_pset_statuses() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server, job, pset,
    };

    let state = DeviceState::new_shared();
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let app = http_server::create_router_with_repositories(
        ObservableState::new(state, broadcaster),
        config::Settings::default(),
        pset::create_default_repository(),
        job::create_default_repository(),
    );

    let missing_pset = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/jobs")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&job_payload(8, 999)).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_pset.status(), StatusCode::NOT_FOUND);

    let mut invalid = job_payload(8, 1);
    invalid["name"] = json!("A name that is longer than twenty-five bytes");
    let invalid_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/jobs")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&invalid).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_response.status(), StatusCode::BAD_REQUEST);

    let restart_missing = app
        .oneshot(
            Request::builder()
                .uri("/jobs/99/restart")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(restart_missing.status(), StatusCode::NOT_FOUND);
}

/// Test POST /config/multi-spindle endpoint (enable)
#[tokio::test]
async fn test_configure_multi_spindle_enable() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server,
    };

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let app = http_server::create_router(observable_state, config::Settings::default());

    let payload = json!({
        "enabled": true,
        "spindle_count": 4,
        "sync_id": 100
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/config/multi-spindle")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse response
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["success"], true);
    assert_eq!(result["enabled"], true);
    assert_eq!(result["spindle_count"], 4);
    assert_eq!(result["sync_id"], 100);

    // Verify state was updated
    let device_state = state.read().unwrap();
    assert!(device_state.multi_spindle_config.enabled);
    assert_eq!(device_state.multi_spindle_config.spindle_count, 4);
}

/// Test POST /config/multi-spindle endpoint (disable)
#[tokio::test]
async fn test_configure_multi_spindle_disable() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server,
    };

    let state = Arc::new(RwLock::new(DeviceState::new()));
    {
        let mut s = state.write().unwrap();
        s.enable_multi_spindle(4, 100).unwrap();
    }

    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let app = http_server::create_router(observable_state, config::Settings::default());

    let payload = json!({
        "enabled": false
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/config/multi-spindle")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse response
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["success"], true);
    assert_eq!(result["enabled"], false);

    // Verify state was updated
    let device_state = state.read().unwrap();
    assert!(!device_state.multi_spindle_config.enabled);
}

/// Test POST /config/multi-spindle endpoint (invalid config)
#[tokio::test]
async fn test_configure_multi_spindle_invalid() {
    use open_protocol_device_simulator::{
        DeviceState, ObservableState, SimulatorEvent, config, http_server,
    };

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let app = http_server::create_router(observable_state, config::Settings::default());

    let payload = json!({
        "enabled": true,
        "spindle_count": 1,  // Too few - invalid
        "sync_id": 100
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/config/multi-spindle")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Parse response
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["success"], false);
    assert_eq!(result["enabled"], false);
}
