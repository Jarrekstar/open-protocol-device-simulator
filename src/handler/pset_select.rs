use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};

/// MID 0018 - Parameter set selection
/// Selects a specific parameter set (pset) for tightening operations
pub struct PsetSelectHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl PsetSelectHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for PsetSelectHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        // Extract pset ID from message data if present
        let pset_str = if !message.data.is_empty() {
            String::from_utf8_lossy(&message.data).to_string()
        } else {
            "1".to_string()
        };

        // Parse pset ID
        let pset_id = pset_str.trim().parse::<u32>().unwrap_or(1);

        println!("MID 0018: Parameter set select - Pset ID: {}", pset_id);

        // Update device state
        {
            let mut state = self.state.write().unwrap();
            state.set_pset(pset_id, Some(format!("Pset_{}", pset_id)));
        }

        // Respond with MID 0016 (Command accepted)
        Ok(Response::new(16, message.revision, Vec::new()))
    }
}
