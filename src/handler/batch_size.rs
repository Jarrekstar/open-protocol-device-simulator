use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};

/// MID 0019 - Set batch size
/// Sets the batch size for the current parameter set
pub struct BatchSizeHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl BatchSizeHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for BatchSizeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        // Extract batch size from message data if present
        let batch_str = if !message.data.is_empty() {
            String::from_utf8_lossy(&message.data).to_string()
        } else {
            "1".to_string()
        };

        // Parse batch size
        let batch_size = batch_str.trim().parse::<u32>().unwrap_or(1);

        println!("MID 0019: Set batch size - Size: {}", batch_size);

        // Update device state
        {
            let mut state = self.state.write().unwrap();
            state.set_batch_size(batch_size);
        }

        // Respond with MID 0005 (Command accepted)
        Ok(Response::new(5, message.revision, Vec::new()))
    }
}
