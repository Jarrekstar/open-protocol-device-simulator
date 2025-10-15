//! MID 0019 - Batch size handler
//!
//! Sets the batch size for the current parameter set. When received, the device
//! transitions to batch mode and tracks tightening progress through the batch.

use crate::handler::data::command_accepted::CommandAccepted;
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
            let pset_id = &message.data[0..=2];
            let batch_size = &message.data[3..];
            (
                String::from_utf8_lossy(pset_id).to_string(),
                String::from_utf8_lossy(batch_size).to_string(),
            )
        } else {
            ("1".to_string(), "1".to_string())
        };

        // Parse batch size
        let batch_size = batch_str.1.trim().parse::<u32>().unwrap_or(1);
        let pset_id = batch_str.0.trim().parse::<u32>().unwrap_or(1);
        println!(
            "MID 0019: Set batch size - PSet: {} -  Size: {}",
            pset_id, batch_size
        );

        // Update device state
        {
            let mut state = self.state.write().unwrap();
            state.set_batch_size(batch_size);
        }

        let ack_data = CommandAccepted::with_mid(19);

        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
