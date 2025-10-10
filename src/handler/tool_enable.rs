use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};

/// MID 0043 - Tool enable
/// Enables the tool for tightening operations
pub struct ToolEnableHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl ToolEnableHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for ToolEnableHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0043: Tool enable request");

        // Update device state
        {
            let mut state = self.state.write().unwrap();
            state.enable_tool();
        }

        // Respond with MID 0005 (Command accepted)
        Ok(Response::new(5, message.revision, Vec::new()))
    }
}
