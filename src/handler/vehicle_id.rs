use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};
use crate::handler::data::command_accepted::CommandAccepted;

/// MID 0052 - Vehicle ID Number download
/// Receives vehicle identification and responds with acknowledgment
pub struct VehicleIdHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl VehicleIdHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for VehicleIdHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        // Extract VIN from message data if present
        let vin = if !message.data.is_empty() {
            String::from_utf8_lossy(&message.data).trim().to_string()
        } else {
            "NO_VIN".to_string()
        };

        println!("MID 0052: Vehicle ID request - VIN: {}", vin);

        // Update device state
        {
            let mut state = self.state.write().unwrap();
            state.set_vehicle_id(vin);
        }

        let ack_data = CommandAccepted::with_mid(52);


        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
