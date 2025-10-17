use crate::handler::data::command_accepted::CommandAccepted;
use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};

/// MID 0050 - Vehicle ID Number download
/// Receives vehicle identification from integrator and responds with acknowledgment
pub struct VehicleIdDownloadHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl VehicleIdDownloadHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for VehicleIdDownloadHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        // Extract VIN from message data if present
        let vin = if !message.data.is_empty() {
            String::from_utf8_lossy(&message.data).trim().to_string()
        } else {
            "NO_VIN".to_string()
        };

        println!("MID 0050: Vehicle ID download - VIN: {}", vin);

        // Update device state
        {
            let mut state = self.state.write().unwrap();
            state.set_vehicle_id(vin);
        }

        let ack_data = CommandAccepted::with_mid(50);

        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
