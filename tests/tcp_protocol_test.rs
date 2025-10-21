mod common;

use open_protocol_device_simulator::{DeviceState, SimulatorEvent, ObservableState, handler, protocol};
use std::sync::{Arc, RwLock};

/// Test MID 0001 - Communication Start
#[test]
fn test_communication_start() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let registry = handler::create_default_registry(observable_state);

    let message = protocol::Message {
        length: 20,
        mid: 1,
        revision: 1,
        data: vec![],
    };

    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 2, "Should respond with MID 0002");
}

/// Test MID 0003 - Communication Stop
#[test]
fn test_communication_stop() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let registry = handler::create_default_registry(observable_state);

    let message = protocol::Message {
        length: 20,
        mid: 3,
        revision: 1,
        data: vec![],
    };

    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(
        response.mid, 5,
        "Should respond with MID 0005 (command accepted)"
    );
}

/// Test MID 9999 - Keep Alive
#[test]
fn test_keep_alive() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let registry = handler::create_default_registry(observable_state);

    let message = protocol::Message {
        length: 20,
        mid: 9999,
        revision: 1,
        data: vec![],
    };

    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 9999, "Should respond with MID 9999");
}

/// Test MID 0018 - Parameter Set Selection
#[test]
fn test_pset_selection() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let registry = handler::create_default_registry(observable_state);

    // Select parameter set 5
    let data = b"005".to_vec();
    let message = protocol::Message {
        length: 23,
        mid: 18,
        revision: 1,
        data,
    };

    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(
        response.mid, 16,
        "Should respond with MID 0016 (pset selected)"
    );

    // Verify state was updated
    let device_state = state.read().unwrap();
    assert_eq!(device_state.current_pset_id, Some(5));
}

/// Test MID 0019 - Batch Size
#[test]
fn test_batch_size() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let registry = handler::create_default_registry(observable_state);

    // Set batch size to 10 for parameter set 1
    let data = b"0010010".to_vec();
    let message = protocol::Message {
        length: 27,
        mid: 19,
        revision: 1,
        data,
    };

    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(
        response.mid, 5,
        "Should respond with MID 0005 (command accepted)"
    );

    // Verify batch size was updated
    let device_state = state.read().unwrap();
    assert_eq!(device_state.tightening_tracker.batch_size(), 10);
}

/// Test MID 0042 - Tool Disable
#[test]
fn test_tool_disable() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let registry = handler::create_default_registry(observable_state);

    let message = protocol::Message {
        length: 20,
        mid: 42,
        revision: 1,
        data: vec![],
    };

    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(
        response.mid, 5,
        "Should respond with MID 0005 (command accepted)"
    );

    // Verify tool was disabled
    let device_state = state.read().unwrap();
    assert!(!device_state.tool_enabled);
}

/// Test MID 0043 - Tool Enable
#[test]
fn test_tool_enable() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    // Disable tool first
    {
        let mut device_state = state.write().unwrap();
        device_state.tool_enabled = false;
    }

    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let registry = handler::create_default_registry(observable_state);

    let message = protocol::Message {
        length: 20,
        mid: 43,
        revision: 1,
        data: vec![],
    };

    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(
        response.mid, 5,
        "Should respond with MID 0005 (command accepted)"
    );

    // Verify tool was enabled
    let device_state = state.read().unwrap();
    assert!(device_state.tool_enabled);
}

/// Test MID 0050 - Vehicle ID Download
#[test]
fn test_vehicle_id_download() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let registry = handler::create_default_registry(observable_state);

    // Download VIN
    let vin = "SSC044207                ";
    let data = vin.as_bytes().to_vec();
    let message = protocol::Message {
        length: 45,
        mid: 50,
        revision: 1,
        data,
    };

    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(
        response.mid, 5,
        "Should respond with MID 0005 (command accepted)"
    );

    // Verify VIN was updated
    let device_state = state.read().unwrap();
    assert_eq!(
        device_state.vehicle_id.as_ref().unwrap().trim(),
        "SSC044207"
    );
}

/// Test MID 0060/0063 - Tightening Result Subscription
#[test]
fn test_tightening_result_subscription() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let registry = handler::create_default_registry(observable_state);

    // Subscribe (MID 0060)
    let message = protocol::Message {
        length: 20,
        mid: 60,
        revision: 1,
        data: vec![],
    };
    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 5, "Should respond with MID 0005");

    // Unsubscribe (MID 0063)
    let message = protocol::Message {
        length: 20,
        mid: 63,
        revision: 1,
        data: vec![],
    };
    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 5, "Should respond with MID 0005");
}

/// Test MID 0014/0017 - Parameter Set Subscription
#[test]
fn test_pset_subscription() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let registry = handler::create_default_registry(observable_state);

    // Subscribe (MID 0014)
    let message = protocol::Message {
        length: 20,
        mid: 14,
        revision: 1,
        data: vec![],
    };
    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 5, "Should respond with MID 0005");

    // Unsubscribe (MID 0017)
    let message = protocol::Message {
        length: 20,
        mid: 17,
        revision: 1,
        data: vec![],
    };
    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 5, "Should respond with MID 0005");
}

/// Test MID 0051/0054 - Vehicle ID Subscription
#[test]
fn test_vehicle_id_subscription() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let registry = handler::create_default_registry(observable_state);

    // Subscribe (MID 0051)
    let message = protocol::Message {
        length: 20,
        mid: 51,
        revision: 1,
        data: vec![],
    };
    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 5, "Should respond with MID 0005");

    // Unsubscribe (MID 0054)
    let message = protocol::Message {
        length: 20,
        mid: 54,
        revision: 1,
        data: vec![],
    };
    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 5, "Should respond with MID 0005");
}

/// Test MID 0090/0092 - Multi-Spindle Status Subscription
#[test]
fn test_multi_spindle_status_subscription() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let registry = handler::create_default_registry(observable_state);

    // Subscribe (MID 0090)
    let message = protocol::Message {
        length: 20,
        mid: 90,
        revision: 1,
        data: vec![],
    };
    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 5, "Should respond with MID 0005");

    // Unsubscribe (MID 0092)
    let message = protocol::Message {
        length: 20,
        mid: 92,
        revision: 1,
        data: vec![],
    };
    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 5, "Should respond with MID 0005");
}

/// Test MID 0100/0103 - Multi-Spindle Result Subscription
#[test]
fn test_multi_spindle_result_subscription() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let registry = handler::create_default_registry(observable_state);

    // Subscribe (MID 0100)
    let message = protocol::Message {
        length: 20,
        mid: 100,
        revision: 1,
        data: vec![],
    };
    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 5, "Should respond with MID 0005");

    // Unsubscribe (MID 0103)
    let message = protocol::Message {
        length: 20,
        mid: 103,
        revision: 1,
        data: vec![],
    };
    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 5, "Should respond with MID 0005");
}

/// Test unknown MID handling
#[test]
fn test_unknown_mid() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let registry = handler::create_default_registry(observable_state);

    let message = protocol::Message {
        length: 20,
        mid: 9998,
        revision: 1,
        data: vec![],
    };

    let result = registry.handle_message(&message);
    assert!(result.is_err(), "Unknown MID should return error");
}

/// Test batch mode lifecycle
#[test]
fn test_batch_lifecycle() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let registry = handler::create_default_registry(observable_state);

    // Set batch size to 3
    let data = b"0010003".to_vec();
    let message = protocol::Message {
        length: 27,
        mid: 19,
        revision: 1,
        data,
    };
    registry
        .handle_message(&message)
        .expect("Handler should succeed");

    // Verify we're in batch mode
    {
        let device_state = state.read().unwrap();
        assert_eq!(device_state.tightening_tracker.batch_size(), 3);
        assert_eq!(device_state.tightening_tracker.counter(), 0);
        assert!(!device_state.tightening_tracker.is_complete());
    }

    // Add 3 tightenings
    for i in 1..=3 {
        let mut device_state = state.write().unwrap();
        device_state.tightening_tracker.add_tightening(true);

        if i < 3 {
            assert!(!device_state.tightening_tracker.is_complete());
        } else {
            assert!(device_state.tightening_tracker.is_complete());
        }
    }
}
