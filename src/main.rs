use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
mod batch_manager;
mod codec;
mod events;
mod handler;
mod http_server;
mod protocol;
mod session;
mod state;
mod subscriptions;

use events::SimulatorEvent;
use state::DeviceState;
use thiserror::Error;
use std::sync::Arc;

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

    // Spawn HTTP server for state inspection and event generation
    let http_state = Arc::clone(&device_state);
    let http_broadcaster = event_tx.clone();
    tokio::spawn(async move {
        http_server::start_http_server(http_state, http_broadcaster).await;
    });

    // Create handler registry (shared across all connections)
    let registry = Arc::new(handler::create_default_registry(device_state));

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Incoming connection from {}", addr);

        let registry = Arc::clone(&registry);
        let mut event_rx = event_tx.subscribe();
        tokio::spawn(async move {
            let codec = codec::null_delimited_codec::NullDelimitedCodec::new();
            let mut framed = tokio_util::codec::Framed::new(stream, codec);

            // Create connection session with typestate pattern
            // Start as Connected (TCP established), will transition to Ready after MID 0001
            let session = session::ConnectionSession::new();
            let session = session.connect(addr);
            let mut session = session.authenticate(); // Immediately ready (simplified for now)

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
                                            16 => session.unsubscribe_pset_selection(),
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
                            SimulatorEvent::PsetChanged { pset_id } => {
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
