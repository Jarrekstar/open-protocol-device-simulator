use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};
use crate::handler::data::command_accepted::CommandAccepted;

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

        let ack_data = CommandAccepted::with_mid(42);


        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
