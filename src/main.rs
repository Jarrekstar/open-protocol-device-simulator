use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
mod codec;
mod handler;
mod http_server;
mod protocol;
mod state;

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

    // Spawn HTTP server for state inspection
    let http_state = Arc::clone(&device_state);
    tokio::spawn(async move {
        http_server::start_http_server(http_state).await;
    });

    // Create handler registry (shared across all connections)
    let registry = Arc::new(handler::create_default_registry(device_state));

    loop {
        let (stream, _) = listener.accept().await?;
        println!("Incoming connection");

        let registry = Arc::clone(&registry);
        tokio::spawn(async move {
            let codec = codec::null_delimited_codec::NullDelimitedCodec::new();
            let mut framed = tokio_util::codec::Framed::new(stream, codec);

            while let Some(result) = framed.next().await {
                match result {
                    Ok(raw_message) => {
                        println!("Received: {:?}", raw_message);

                        // Parse the message
                        match protocol::parser::parse_message(&raw_message) {
                            Ok(message) => {
                                println!("Parsed MID {}, revision {}", message.mid, message.revision);

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
                                        // Optionally send error response (MID 0004)
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
            // This runs when the loop exits (disconnect)
            println!("Client disconnected");
        });
    }
}

#[derive(Error, Debug)]
pub enum ServeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
