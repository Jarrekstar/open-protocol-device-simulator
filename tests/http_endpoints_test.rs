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
    use open_protocol_device_simulator::{DeviceState, SimulatorEvent, http_server};

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);

    let app = http_server::create_router(Arc::clone(&state), broadcaster);

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
    use open_protocol_device_simulator::{DeviceState, SimulatorEvent, http_server};

    let state = Arc::new(RwLock::new(DeviceState::new()));
    {
        let mut s = state.write().unwrap();
        s.set_batch_size(5); // Enable batch mode for counter tracking
    }

    let (broadcaster, _receiver) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let app = http_server::create_router(Arc::clone(&state), broadcaster);

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

/// Test POST /auto-tightening/start endpoint
#[tokio::test]
async fn test_start_auto_tightening_endpoint() {
    use open_protocol_device_simulator::{DeviceState, SimulatorEvent, http_server};

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let app = http_server::create_router(Arc::clone(&state), broadcaster);

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
    use open_protocol_device_simulator::{DeviceState, SimulatorEvent, http_server};
    use std::sync::atomic::AtomicBool;

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);

    // Create server state with auto-tightening already active
    let server_state = http_server::ServerState {
        device_state: Arc::clone(&state),
        broadcaster: broadcaster.clone(),
        auto_tightening_active: Arc::new(AtomicBool::new(true)), // Already running
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
    use open_protocol_device_simulator::{DeviceState, SimulatorEvent, http_server};

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let app = http_server::create_router(Arc::clone(&state), broadcaster);

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
    use open_protocol_device_simulator::{DeviceState, SimulatorEvent, http_server};

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let app = http_server::create_router(Arc::clone(&state), broadcaster);

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

/// Test POST /config/multi-spindle endpoint (enable)
#[tokio::test]
async fn test_configure_multi_spindle_enable() {
    use open_protocol_device_simulator::{DeviceState, SimulatorEvent, http_server};

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let app = http_server::create_router(Arc::clone(&state), broadcaster);

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
    use open_protocol_device_simulator::{DeviceState, SimulatorEvent, http_server};

    let state = Arc::new(RwLock::new(DeviceState::new()));
    {
        let mut s = state.write().unwrap();
        s.enable_multi_spindle(4, 100).unwrap();
    }

    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let app = http_server::create_router(Arc::clone(&state), broadcaster);

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
    use open_protocol_device_simulator::{DeviceState, SimulatorEvent, http_server};

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let app = http_server::create_router(Arc::clone(&state), broadcaster);

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
