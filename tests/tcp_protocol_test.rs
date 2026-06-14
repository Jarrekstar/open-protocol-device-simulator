mod common;

use open_protocol_device_simulator::protocol::revision::{ProtocolConfiguration, RevisionPolicy};
use open_protocol_device_simulator::{
    DeviceState, ObservableState, SimulatorEvent, handler, protocol,
};
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

#[test]
fn test_disabled_revision_returns_error_97() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let registry = handler::create_default_registry(observable_state);

    let response = registry
        .handle_message(&protocol::Message {
            length: 20,
            mid: 1,
            revision: 2,
            data: vec![],
        })
        .unwrap();

    assert_eq!(response.mid, 4);
    assert_eq!(response.revision, 1);
    assert_eq!(response.data, b"000197");
}

#[test]
fn test_communication_start_revision_six_uses_configured_samples() {
    use open_protocol_device_simulator::ProtocolConfiguration;

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let protocol_configuration = ProtocolConfiguration::default();
    let mut profile = protocol_configuration.profile();
    profile.families.get_mut("communication").unwrap().revision = 6;
    profile.samples.supports_sequence_number = true;
    profile.samples.supports_message_linking = true;
    profile.samples.station_id = 7;
    protocol_configuration.update(profile).unwrap();

    let registry = handler::create_registry_with_repositories_and_protocol(
        observable_state,
        open_protocol_device_simulator::pset::create_default_repository(),
        open_protocol_device_simulator::job::create_default_repository(),
        protocol_configuration,
    );
    let response = registry
        .handle_message(&protocol::Message {
            length: 20,
            mid: 1,
            revision: 6,
            data: vec![],
        })
        .unwrap();

    assert_eq!(response.mid, 2);
    assert_eq!(response.revision, 6);
    let data = String::from_utf8(response.data).unwrap();
    assert!(data.contains("052.8.0"));
    assert!(data.contains("121"));
    assert!(data.contains("131"));
    assert!(data.ends_with("1407"));
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
    let mut event_rx = broadcaster.subscribe();
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let registry = handler::create_default_registry(observable_state);

    // Select PSET 002 first, then configure its batch via MID 0019 (PPPBB:
    // PSET 002, batch size 20). Setting a batch size is a runtime Pset
    // attribute, so it is accepted straight from the Pset profile and
    // switches the device to batch counting.
    let select = protocol::parser::parse_message(b"00230018001         002").unwrap();
    let response = registry
        .handle_message(&select)
        .expect("Handler should succeed");
    assert_eq!(response.mid, 16);

    let raw = b"00250019001         00220";
    let message = protocol::parser::parse_message(raw).expect("MID 0019 frame should parse");

    let response = registry
        .handle_message(&message)
        .expect("Handler should succeed");
    assert_eq!(
        response.mid, 5,
        "Should respond with MID 0005 (command accepted)"
    );

    // Verify batch size was updated and the profile followed
    let device_state = state.read().unwrap();
    assert_eq!(device_state.tightening_tracker.batch_size(), 20);
    assert_eq!(
        device_state.operation_mode(),
        open_protocol_device_simulator::OperationMode::Batch
    );

    // The Pset -> Batch transition must be broadcast so the web UI updates
    let events = std::iter::from_fn(|| event_rx.try_recv().ok()).collect::<Vec<_>>();
    assert!(events.iter().any(|event| matches!(
        event,
        SimulatorEvent::OperationModeChanged {
            mode: open_protocol_device_simulator::OperationMode::Batch
        }
    )));
}

/// MID 0019/0020 target a specific PSET: unknown IDs are rejected, sizes for
/// non-selected PSETs are stored without touching the running tracker, and
/// only the running PSET's counter can be reset.
#[test]
fn test_batch_commands_respect_target_pset() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let registry = handler::create_default_registry(observable_state);

    // PSET 999 does not exist: error 02 "Parameter set ID not present"
    let unknown = protocol::parser::parse_message(b"00250019001         99920").unwrap();
    let response = registry.handle_message(&unknown).unwrap();
    assert_eq!(response.mid, 4);
    assert_eq!(response.data, b"001902");

    // PSET 2 exists but PSET 1 is selected: accepted, but the running
    // tracker is untouched until PSET 2 gets selected.
    let stored = protocol::parser::parse_message(b"00250019001         00220").unwrap();
    let response = registry.handle_message(&stored).unwrap();
    assert_eq!(response.mid, 5);
    assert_eq!(state.read().unwrap().tightening_tracker.batch_size(), 0);
    assert_eq!(
        state.read().unwrap().operation_mode(),
        open_protocol_device_simulator::OperationMode::Pset
    );

    // Selecting PSET 2 activates its stored batch size
    let select = protocol::parser::parse_message(b"00230018001         002").unwrap();
    let response = registry.handle_message(&select).unwrap();
    assert_eq!(response.mid, 16);
    assert_eq!(state.read().unwrap().tightening_tracker.batch_size(), 20);
    assert_eq!(
        state.read().unwrap().operation_mode(),
        open_protocol_device_simulator::OperationMode::Batch
    );

    // MID 0020 with malformed data: error 01 "Invalid data"
    let malformed = protocol::parser::parse_message(b"00230020001         abc").unwrap();
    let response = registry.handle_message(&malformed).unwrap();
    assert_eq!(response.mid, 4);
    assert_eq!(response.data, b"002001");

    // MID 0020 for a PSET that is not running: error 04
    let not_running = protocol::parser::parse_message(b"00230020001         001").unwrap();
    let response = registry.handle_message(&not_running).unwrap();
    assert_eq!(response.mid, 4);
    assert_eq!(response.data, b"002004");

    // MID 0020 for the running PSET resets the counter
    let reset = protocol::parser::parse_message(b"00230020001         002").unwrap();
    let response = registry.handle_message(&reset).unwrap();
    assert_eq!(response.mid, 5);
}

#[test]
fn test_pset_selected_acknowledgement_has_no_response() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let registry = handler::create_default_registry(observable_state);
    let message =
        protocol::parser::parse_message(b"002000160011        ").expect("MID 0016 should parse");
    let mut subscriptions = open_protocol_device_simulator::subscriptions::Subscriptions::new();

    assert!(matches!(
        registry.dispatch(&message, &mut subscriptions),
        Ok(handler::HandlerResult::NoResponse)
    ));
}

#[test]
fn pset_lifecycle_is_available_in_batch_mode() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    state.write().unwrap().set_batch_mode();
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let registry = handler::create_default_registry(observable_state);
    let mut subscriptions = open_protocol_device_simulator::subscriptions::Subscriptions::new();

    let subscribe = protocol::parser::parse_message(b"00200014001         ").unwrap();
    let response = registry
        .dispatch(&subscribe, &mut subscriptions)
        .expect("MID 0014 should be accepted in Batch mode");
    assert!(matches!(
        response,
        handler::HandlerResult::Response(response) if response.mid == 5
    ));

    let select = protocol::parser::parse_message(b"00230018001         002").unwrap();
    let response = registry
        .dispatch(&select, &mut subscriptions)
        .expect("MID 0018 should be accepted in Batch mode");
    assert!(matches!(
        response,
        handler::HandlerResult::Response(response) if response.mid == 16
    ));

    let acknowledge =
        protocol::parser::parse_message(b"002000160011        ").expect("MID 0016 should parse");
    assert!(matches!(
        registry.dispatch(&acknowledge, &mut subscriptions),
        Ok(handler::HandlerResult::NoResponse)
    ));
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

#[test]
fn all_configured_revisions_for_existing_mid_families_are_dispatchable() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    state.write().unwrap().set_job_mode();
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(state, broadcaster);
    let configuration = ProtocolConfiguration::default();
    let mut profile = configuration.profile();
    for selection in profile.families.values_mut() {
        selection.policy = RevisionPolicy::AnyImplemented;
    }
    configuration.update(profile).unwrap();
    let registry = handler::create_registry_with_repositories_and_protocol(
        observable_state,
        open_protocol_device_simulator::pset::create_default_repository(),
        open_protocol_device_simulator::job::create_default_repository(),
        configuration,
    );

    let response = registry
        .handle_message(&protocol::Message {
            length: 20,
            mid: 30,
            revision: 2,
            data: Vec::new(),
        })
        .unwrap();
    assert_eq!(response.mid, 31);
    assert_eq!(response.revision, 2);

    for revision in 1..=5 {
        let response = registry
            .handle_message(&protocol::Message {
                length: 20,
                mid: 34,
                revision,
                data: Vec::new(),
            })
            .unwrap();
        assert_eq!(response.mid, 5);
    }

    for revision in [1, 2, 3, 4, 5, 6, 7, 998, 999] {
        let response = registry
            .handle_message(&protocol::Message {
                length: 20,
                mid: 60,
                revision,
                data: Vec::new(),
            })
            .unwrap();
        assert_eq!(response.mid, 5);
    }

    for revision in 1..=5 {
        let data = match revision {
            1 => Vec::new(),
            2 => b"0000000000".to_vec(),
            _ => b"00000000001".to_vec(),
        };
        let response = registry
            .handle_message(&protocol::Message {
                length: 20 + data.len() as u32,
                mid: 100,
                revision,
                data,
            })
            .unwrap();
        assert_eq!(response.mid, 5);
    }
}

/// MID 0127 - Abort Job stops a running Job and acknowledges with MID 0005.
#[test]
fn test_abort_job_stops_running_job() {
    use open_protocol_device_simulator::OperationMode;
    use open_protocol_device_simulator::job::{Job, JobStep};

    let state = Arc::new(RwLock::new(DeviceState::new()));
    // A Job can only be selected from the Job profile (MID 0038 is otherwise
    // rejected), so the operator switches the controller into Job mode first.
    state.write().unwrap().set_job_mode();
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let mut event_rx = broadcaster.subscribe();
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);

    let psets = open_protocol_device_simulator::pset::create_default_repository();
    let jobs = open_protocol_device_simulator::job::create_default_repository();
    jobs.write()
        .unwrap()
        .create(Job {
            id: 7,
            name: "Abortable".to_string(),
            forced_order: 1,
            first_tightening_timeout: 0,
            job_timeout: 0,
            batch_count_mode: 0,
            lock_at_job_done: false,
            use_line_control: false,
            repeat_job: false,
            loosening_mode: 0,
            repair_mode: 0,
            steps: vec![JobStep {
                channel_id: 1,
                pset_id: 1,
                auto_value: true,
                batch_size: 3,
            }],
        })
        .unwrap();
    let registry = handler::create_registry_with_repositories(observable_state, psets, jobs);

    // Select Job 07 (two-digit revision 1 codec) so a Job is running.
    let select = protocol::Message {
        length: 22,
        mid: 38,
        revision: 1,
        data: b"07".to_vec(),
    };
    let response = registry.handle_message(&select).expect("Job select");
    assert_eq!(response.mid, 5, "MID 0038 should be accepted");
    assert!(state.read().unwrap().is_job_running());

    // Abort the running Job (MID 0127).
    let abort = protocol::Message {
        length: 20,
        mid: 127,
        revision: 1,
        data: vec![],
    };
    let response = registry.handle_message(&abort).expect("Abort Job");
    assert_eq!(response.mid, 5, "Abort Job must answer MID 0005");
    assert_eq!(response.data, b"0127", "ACK echoes the aborted MID");

    {
        let device = state.read().unwrap();
        assert!(!device.is_job_mode());
        assert!(!device.is_job_running());
        assert_eq!(device.current_job_id, None);
        assert_eq!(device.operation_mode(), OperationMode::Pset);
    }

    let events = std::iter::from_fn(|| event_rx.try_recv().ok()).collect::<Vec<_>>();
    assert!(
        events.iter().any(
            |event| matches!(event, SimulatorEvent::JobAborted { state } if state.job_id == 7)
        ),
        "abort should broadcast JobAborted, got {events:?}"
    );
}

/// MID 0127 - Abort Job is accepted in Pset mode and acknowledges even when no
/// Job is running (no error reply is defined for Abort Job).
#[test]
fn test_abort_job_with_no_running_job_in_pset_mode() {
    use open_protocol_device_simulator::OperationMode;

    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let registry = handler::create_default_registry(observable_state);

    // Fresh device starts in the Pset profile with no Job loaded.
    assert_eq!(state.read().unwrap().operation_mode(), OperationMode::Pset);

    let abort = protocol::Message {
        length: 20,
        mid: 127,
        revision: 1,
        data: vec![],
    };
    let response = registry.handle_message(&abort).expect("Abort Job");
    assert_eq!(response.mid, 5, "Abort Job must answer MID 0005");
    assert_eq!(response.data, b"0127", "ACK echoes the aborted MID");

    let device = state.read().unwrap();
    assert_eq!(device.operation_mode(), OperationMode::Pset);
    assert_eq!(device.current_job_id, None);
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

    let response = registry
        .handle_message(&message)
        .expect("Unknown MID should produce MID 0004");
    assert_eq!(response.mid, 4);
    assert_eq!(response.data, b"999899");
}

/// Test batch mode lifecycle
#[test]
fn test_batch_lifecycle() {
    let state = Arc::new(RwLock::new(DeviceState::new()));
    let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);
    let observable_state = ObservableState::new(Arc::clone(&state), broadcaster);
    let registry = handler::create_default_registry(observable_state);
    state.write().unwrap().set_batch_mode();

    // Set batch size to 3
    let data = b"00103".to_vec();
    let message = protocol::Message {
        length: 25,
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
