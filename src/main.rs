use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use open_protocol_device_simulator::{
    codec, events, handler, http_server, observable_state, protocol, session, state,
};
use std::sync::Arc;
use thiserror::Error;

use events::SimulatorEvent;
use observable_state::ObservableState;
use state::DeviceState;

#[tokio::main]
async fn main() {
    serve_tcp_client().await.unwrap();
}

async fn serve_tcp_client() -> Result<(), ServeError> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

    // Create device state (shared across all connections)
    let device_state = DeviceState::new_shared();

    // Create event broadcast channel (capacity of 100 events)
    let (event_tx, _event_rx) = tokio::sync::broadcast::channel::<SimulatorEvent>(100);

    // Create observable state wrapper that broadcasts events on state changes
    let observable_state = ObservableState::new(device_state, event_tx.clone());

    // Spawn HTTP server for state inspection and event generation
    let http_observable = observable_state.clone();
    tokio::spawn(async move {
        http_server::start_http_server(http_observable).await;
    });

    // Create handler registry (shared across all connections)
    let registry = Arc::new(handler::create_default_registry(observable_state));

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Incoming connection from {}", addr);

        let registry = Arc::clone(&registry);
        let mut event_rx = event_tx.subscribe();
        tokio::spawn(async move {
            let codec = codec::null_delimited_codec::NullDelimitedCodec::new();
            let mut framed = tokio_util::codec::Framed::new(stream, codec);

            // Create connection session with typestate pattern
            // Transitions: Disconnected → Connected → Ready
            let session = session::ConnectionSession::new();
            let session = session.connect(addr);
            let mut session = session.authenticate(); // Immediate transition to Ready state

            loop {
                tokio::select! {
                    // Handle incoming TCP messages (requests from client)
                    Some(result) = framed.next() => {
                        match result {
                            Ok(raw_message) => {
                                println!("Received: {:?}", raw_message);

                                // Update keep-alive timestamp
                                session.update_keep_alive();

                                // Parse the message
                                match protocol::parser::parse_message(&raw_message) {
                                    Ok(message) => {
                                        println!("Parsed MID {}, revision {}", message.mid, message.revision);

                                        // Track subscription state based on MID using session
                                        match message.mid {
                                            60 => session.subscribe_tightening_result(),
                                            63 => session.unsubscribe_tightening_result(),
                                            14 => session.subscribe_pset_selection(),
                                            17 => session.unsubscribe_pset_selection(),
                                            51 => session.subscribe_vehicle_id(),
                                            54 => session.unsubscribe_vehicle_id(),
                                            90 => session.subscribe_multi_spindle_status(),
                                            92 => session.unsubscribe_multi_spindle_status(),
                                            100 => session.subscribe_multi_spindle_result(),
                                            103 => session.unsubscribe_multi_spindle_result(),
                                            _ => {}
                                        }

                                        // Handle the message
                                        match registry.handle_message(&message) {
                                            Ok(response) => {
                                                // Serialize and send response
                                                let response_bytes = protocol::serializer::serialize_response(&response);
                                                println!("Sending response: MID {}", response.mid);

                                                if let Err(e) = framed.send(response_bytes.as_slice().into()).await {
                                                    eprintln!("send error: {e}");
                                                    break;
                                                }

                                                // Special handling for MID 51 (vehicle ID subscription)
                                                // Send VIN immediately after subscription is confirmed
                                                if message.mid == 51 {
                                                    // VIN is empty because handlers don't have direct state access
                                                    // VIN changes are broadcast via SimulatorEvent::VehicleIdChanged
                                                    let current_vin = String::new();
                                                    let vin_data = handler::data::VehicleIdBroadcast::new(current_vin.clone());
                                                    let vin_response = protocol::Response::from_data(52, 1, vin_data);
                                                    let vin_response_bytes = protocol::serializer::serialize_response(&vin_response);
                                                    println!("Sending initial MID 0052 with current VIN: {}", current_vin);

                                                    if let Err(e) = framed.send(vin_response_bytes.as_slice().into()).await {
                                                        eprintln!("send error during initial VIN broadcast: {e}");
                                                        break;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Handler error: {e}");
                                                // Send error response (MID 0004)
                                                let error_response = handler::data::ErrorResponse::generic(message.mid);
                                                let response = protocol::Response::from_data(4, message.revision, error_response);
                                                let response_bytes = protocol::serializer::serialize_response(&response);
                                                println!("Sending error response: MID 0004 for failed MID {}", message.mid);

                                                if let Err(e) = framed.send(response_bytes.as_slice().into()).await {
                                                    eprintln!("send error: {e}");
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Parse error: {e}");
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("framed read error: {e}");
                                break;
                            }
                        }
                    }

                    // Handle broadcast events (push notifications)
                    Ok(event) = event_rx.recv() => {
                        match event {
                            SimulatorEvent::TighteningCompleted { result } => {
                                if session.subscriptions().is_subscribed_to_tightening_result() {
                                    println!("Broadcasting MID 0061 to subscribed client ({})", session.addr());
                                    let response = protocol::Response::from_data(61, 1, result);
                                    let response_bytes = protocol::serializer::serialize_response(&response);

                                    if let Err(e) = framed.send(response_bytes.as_slice().into()).await {
                                        eprintln!("send error during broadcast: {e}");
                                        break;
                                    }
                                }
                            }
                            SimulatorEvent::PsetChanged { pset_id, pset_name: _ } => {
                                if session.subscriptions().is_subscribed_to_pset_selection() {
                                    println!("Broadcasting MID 0015 to subscribed client ({}): pset {}", session.addr(), pset_id);
                                    let pset_data = handler::data::PsetSelected::new(pset_id);
                                    let response = protocol::Response::from_data(15, 1, pset_data);
                                    let response_bytes = protocol::serializer::serialize_response(&response);

                                    if let Err(e) = framed.send(response_bytes.as_slice().into()).await {
                                        eprintln!("send error during broadcast: {e}");
                                        break;
                                    }
                                }
                            }
                            SimulatorEvent::ToolStateChanged { enabled } => {
                                println!("Tool state changed: {}", if enabled { "enabled" } else { "disabled" });
                                // No standard MID for tool state broadcasts in Open Protocol
                            }
                            SimulatorEvent::BatchCompleted { total } => {
                                println!("Batch completed: {} tightenings", total);
                                // Could send MID 0061 with batch status if subscribed
                            }
                            SimulatorEvent::VehicleIdChanged { vin } => {
                                if session.subscriptions().is_subscribed_to_vehicle_id() {
                                    println!("Broadcasting MID 0052 to subscribed client ({}): VIN {}", session.addr(), vin);
                                    let vin_data = handler::data::VehicleIdBroadcast::new(vin);
                                    let response = protocol::Response::from_data(52, 1, vin_data);
                                    let response_bytes = protocol::serializer::serialize_response(&response);

                                    if let Err(e) = framed.send(response_bytes.as_slice().into()).await {
                                        eprintln!("send error during broadcast: {e}");
                                        break;
                                    }
                                }
                            }
                            SimulatorEvent::MultiSpindleStatusCompleted { status } => {
                                if session.subscriptions().is_subscribed_to_multi_spindle_status() {
                                    println!("Broadcasting MID 0091 to subscribed client ({}): sync_id {}, status {}",
                                        session.addr(), status.sync_id, status.status);
                                    let status_data = handler::data::MultiSpindleStatusBroadcast::new(status);
                                    let response = protocol::Response::from_data(91, 1, status_data);
                                    let response_bytes = protocol::serializer::serialize_response(&response);

                                    if let Err(e) = framed.send(response_bytes.as_slice().into()).await {
                                        eprintln!("send error during broadcast: {e}");
                                        break;
                                    }
                                }
                            }
                            SimulatorEvent::MultiSpindleResultCompleted { result } => {
                                if session.subscriptions().is_subscribed_to_multi_spindle_result() {
                                    println!("Broadcasting MID 0101 to subscribed client ({}): result_id {}, sync_id {}, status {}",
                                        session.addr(), result.result_id, result.sync_id,
                                        if result.is_ok() { "OK" } else { "NOK" });

                                    // Create MID 0101 broadcast with multi-spindle result data
                                    let result_data = handler::data::MultiSpindleResultBroadcast::new(
                                        result,
                                        String::new(), // VIN (not available in session context)
                                        1,             // job_id
                                        1,             // pset_id
                                        0,             // batch_size
                                        0,             // batch_counter
                                        2,             // batch_status
                                    );
                                    let response = protocol::Response::from_data(101, 1, result_data);
                                    let response_bytes = protocol::serializer::serialize_response(&response);

                                    if let Err(e) = framed.send(response_bytes.as_slice().into()).await {
                                        eprintln!("send error during broadcast: {e}");
                                        break;
                                    }
                                }
                            }
                            SimulatorEvent::AutoTighteningProgress { .. } => {
                                // Auto-tightening progress is only sent to WebSocket clients, not TCP
                                // No MID exists in Open Protocol for auto-tightening progress
                            }
                        }
                    }
                }
            }
            // This runs when the loop exits (disconnect)
            println!("Client disconnected: {}", session.addr());
            drop(session); // Explicitly drop to clean up resources
        });
    }
}

#[derive(Error, Debug)]
pub enum ServeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
