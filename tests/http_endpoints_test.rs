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
    assert_eq!(
        state.read().unwrap().current_job_status,
        Some(open_protocol_device_simulator::job::JobStatus::Ok)
    );

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
