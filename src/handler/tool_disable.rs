use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};

/// MID 0042 - Tool disable
/// Disables the tool to prevent tightening operations
pub struct ToolDisableHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl ToolDisableHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for ToolDisableHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0042: Tool disable request");

        // Update device state
        {
            let mut state = self.state.write().unwrap();
            state.disable_tool();
        }

        // Respond with MID 0005 (Command accepted)
        Ok(Response::new(5, message.revision, Vec::new()))
    }
}
