mod common;

use std::sync::{Arc, RwLock};

/// Test device state initialization
#[tokio::test]
async fn test_device_state_initialization() {
    use open_protocol_device_simulator::DeviceState;

    let state = DeviceState::new();

    // Verify initial state
    assert_eq!(state.cell_id, 1);
    assert_eq!(state.channel_id, 1);
    assert_eq!(state.controller_name, "OpenProtocolSimulator");
    assert!(state.tool_enabled);
    assert_eq!(state.current_pset_id, Some(1));
    assert_eq!(state.current_job_id, Some(1));
}

/// Test multi-spindle configuration
#[tokio::test]
async fn test_multi_spindle_configuration() {
    use open_protocol_device_simulator::DeviceState;

    let mut state = DeviceState::new();

    // Enable multi-spindle mode
    let result = state.enable_multi_spindle(4, 100);
    assert!(result.is_ok());
    assert!(state.multi_spindle_config.enabled);
    assert_eq!(state.multi_spindle_config.spindle_count, 4);
    assert_eq!(state.multi_spindle_config.sync_id, 100);

    // Disable multi-spindle mode
    state.disable_multi_spindle();
    assert!(!state.multi_spindle_config.enabled);
    assert_eq!(state.multi_spindle_config.spindle_count, 1);
}

/// Test invalid multi-spindle configuration
#[tokio::test]
async fn test_invalid_multi_spindle_config() {
    use open_protocol_device_simulator::DeviceState;

    let mut state = DeviceState::new();

    // Try to enable with invalid spindle count (too few)
    let result = state.enable_multi_spindle(1, 100);
    assert!(result.is_err());

    // Try to enable with invalid spindle count (too many)
    let result = state.enable_multi_spindle(17, 100);
    assert!(result.is_err());

    // Verify state wasn't changed
    assert!(!state.multi_spindle_config.enabled);
}

/// Test tightening simulation with event broadcasting
#[tokio::test]
async fn test_tightening_simulation() {
    use open_protocol_device_simulator::{DeviceState, SimulatorEvent, handler::data::TighteningResult};

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, mut receiver) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);

    // Set batch mode first to track counter
    {
        let mut s = state.write().unwrap();
        s.set_batch_size(5);
    }

    // Simulate a tightening by creating a result and broadcasting
    let tightening_info = {
        let mut s = state.write().unwrap();
        let info = s.tightening_tracker.add_tightening(true);

        // Verify tightening was tracked
        assert_eq!(info.counter, 1);
        assert_eq!(info.tightening_id, 1);

        info
    };

    // Verify counter incremented
    let counter = {
        let s = state.read().unwrap();
        s.tightening_tracker.counter()
    };
    assert_eq!(counter, 1);

    // Create and broadcast a tightening result event
    let result = TighteningResult {
        cell_id: 1,
        channel_id: 1,
        controller_name: "Test".to_string(),
        vin_number: None,
        job_id: 1,
        pset_id: 1,
        batch_size: 5,
        batch_counter: tightening_info.counter as u32,
        tightening_status: true,
        torque_status: true,
        angle_status: true,
        torque_min: 10.0,
        torque_max: 15.0,
        torque_target: 12.5,
        torque: 12.5,
        angle_min: 30.0,
        angle_max: 50.0,
        angle_target: 40.0,
        angle: 40.0,
        timestamp: "2024-01-01 12:00:00".to_string(),
        last_pset_change: None,
        batch_status: None,
        tightening_id: Some(tightening_info.tightening_id as u32),
    };

    // Broadcast the tightening completed event
    let _ = broadcaster.send(SimulatorEvent::TighteningCompleted {
        result: result.clone(),
    });

    // Verify event was received
    let event = receiver.recv().await.unwrap();
    match event {
        SimulatorEvent::TighteningCompleted { result: received_result } => {
            assert_eq!(received_result.batch_counter, 1);
            assert_eq!(received_result.tightening_id, Some(1));
            assert!(received_result.tightening_status);
        }
        _ => panic!("Expected TighteningCompleted event"),
    }
}

/// Test batch mode tracking
#[tokio::test]
async fn test_batch_mode_tracking() {
    use open_protocol_device_simulator::DeviceState;

    let mut state = DeviceState::new();

    // Set batch size
    state.set_batch_size(5);
    assert_eq!(state.tightening_tracker.batch_size(), 5);
    assert!(!state.tightening_tracker.is_complete());

    // Add tightenings
    for i in 1..=5 {
        let info = state.tightening_tracker.add_tightening(true);
        assert_eq!(info.counter, i);

        if i < 5 {
            assert!(!state.tightening_tracker.is_complete());
        } else {
            assert!(state.tightening_tracker.is_complete());
        }
    }
}

/// Test tool enable/disable state management
#[tokio::test]
async fn test_tool_state_management() {
    use open_protocol_device_simulator::DeviceState;

    let mut state = DeviceState::new();

    // Tool should be enabled by default
    assert!(state.tool_enabled);

    // Disable tool
    state.tool_enabled = false;
    assert!(!state.tool_enabled);

    // Re-enable tool
    state.tool_enabled = true;
    assert!(state.tool_enabled);
}

/// Test parameter set selection
#[tokio::test]
async fn test_pset_selection() {
    use open_protocol_device_simulator::DeviceState;

    let mut state = DeviceState::new();

    // Default pset is 1
    assert_eq!(state.current_pset_id, Some(1));

    // Select different pset
    state.set_pset(5, Some("Custom Pset".to_string()));
    assert_eq!(state.current_pset_id, Some(5));
    assert_eq!(state.current_pset_name, Some("Custom Pset".to_string()));

    // Select another pset without name
    state.set_pset(10, None);
    assert_eq!(state.current_pset_id, Some(10));
    assert_eq!(state.current_pset_name, None);
}

/// Test vehicle ID management
#[tokio::test]
async fn test_vehicle_id_management() {
    use open_protocol_device_simulator::{DeviceState, SimulatorEvent};

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, mut receiver) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);

    // Set vehicle ID
    {
        let mut s = state.write().unwrap();
        s.vehicle_id = Some("TEST12345".to_string());
    }

    // Broadcast vehicle ID change
    let vin = {
        let s = state.read().unwrap();
        s.vehicle_id.clone().unwrap()
    };

    let _ = broadcaster.send(SimulatorEvent::VehicleIdChanged { vin: vin.clone() });

    // Verify event was broadcast
    let event = receiver.recv().await.unwrap();
    match event {
        SimulatorEvent::VehicleIdChanged { vin: received_vin } => {
            assert_eq!(received_vin, "TEST12345");
        }
        _ => panic!("Expected VehicleIdChanged event"),
    }
}

/// Test event broadcasting system
#[tokio::test]
async fn test_event_broadcasting() {
    use open_protocol_device_simulator::{SimulatorEvent, handler::data::TighteningResult};

    let (broadcaster, mut receiver1) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let mut receiver2 = broadcaster.subscribe();

    // Create a mock tightening result
    let result = TighteningResult {
        cell_id: 1,
        channel_id: 1,
        controller_name: "Test".to_string(),
        vin_number: None,
        job_id: 1,
        pset_id: 1,
        batch_size: 1,
        batch_counter: 1,
        tightening_status: true,
        torque_status: true,
        angle_status: true,
        torque_min: 10.0,
        torque_max: 15.0,
        torque_target: 12.5,
        torque: 12.5,
        angle_min: 30.0,
        angle_max: 50.0,
        angle_target: 40.0,
        angle: 40.0,
        timestamp: "2024-01-01 12:00:00".to_string(),
        last_pset_change: None,
        batch_status: None,
        tightening_id: Some(1),
    };

    // Broadcast event
    let _ = broadcaster.send(SimulatorEvent::TighteningCompleted {
        result: result.clone(),
    });

    // Both subscribers should receive it
    let event1 = receiver1.recv().await.unwrap();
    let event2 = receiver2.recv().await.unwrap();

    match (event1, event2) {
        (
            SimulatorEvent::TighteningCompleted { .. },
            SimulatorEvent::TighteningCompleted { .. },
        ) => {
            // Success
        }
        _ => panic!("Expected TighteningCompleted events"),
    }
}

/// Test batch completion tracking
#[tokio::test]
async fn test_batch_completion() {
    use open_protocol_device_simulator::DeviceState;

    let mut state = DeviceState::new();

    // Set batch size to 3
    state.set_batch_size(3);

    // Add 2 tightenings
    state.tightening_tracker.add_tightening(true);
    state.tightening_tracker.add_tightening(true);
    assert!(!state.tightening_tracker.is_complete());

    // Add final tightening
    state.tightening_tracker.add_tightening(true);
    assert!(state.tightening_tracker.is_complete());

    // Verify we should wait for new config
    assert!(state.tightening_tracker.should_wait_for_config());
}

/// Test multi-spindle result generation
#[tokio::test]
async fn test_multi_spindle_result_generation() {
    use open_protocol_device_simulator::multi_spindle::{
        MultiSpindleConfig, generate_multi_spindle_results,
    };

    let config = MultiSpindleConfig::new(4, 100);
    let result = generate_multi_spindle_results(&config, 1, 1);

    // Verify result structure
    assert_eq!(result.spindle_count, 4);
    assert_eq!(result.spindle_results.len(), 4);
    assert_eq!(result.sync_id, 100);
    assert_eq!(result.result_id, 1);

    // Verify spindle IDs are sequential
    for (idx, spindle) in result.spindle_results.iter().enumerate() {
        assert_eq!(spindle.spindle_id, (idx + 1) as u8);
        assert_eq!(spindle.channel_id, (idx + 1) as u8);
    }
}

/// Test FSM state transitions
#[tokio::test]
async fn test_fsm_transitions() {
    use open_protocol_device_simulator::device_fsm::{DeviceFSM, TighteningParams};

    // Start with idle FSM
    let fsm = DeviceFSM::new();

    // Transition to tightening
    let params = TighteningParams::default_test();
    let fsm = fsm.start_tightening(params);

    // Complete tightening
    let fsm = fsm.complete();

    // Get result
    let outcome = fsm.result();
    assert!(outcome.actual_torque > 0.0);
    assert!(outcome.actual_angle > 0.0);
}

/// Test protocol message parsing
#[tokio::test]
async fn test_protocol_message_parsing() {
    use open_protocol_device_simulator::protocol;

    // Valid message: length=20, MID=1, revision=1
    let raw = b"00200001001         ";
    let message = protocol::parser::parse_message(raw).unwrap();

    assert_eq!(message.length, 20);
    assert_eq!(message.mid, 1);
    assert_eq!(message.revision, 1);
    assert!(message.data.is_empty());
}

/// Test protocol message serialization
#[tokio::test]
async fn test_protocol_serialization() {
    use open_protocol_device_simulator::protocol::{Response, serializer};

    let response = Response::new(2, 1, vec![]);
    let serialized = serializer::serialize_response(&response);

    // Should be exactly 20 bytes for header-only message
    assert_eq!(serialized.len(), 20);

    // Verify format
    let as_string = String::from_utf8_lossy(&serialized);
    assert!(as_string.starts_with("0020"));
    assert!(as_string.contains("0002"));
}
