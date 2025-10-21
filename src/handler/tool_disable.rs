//! MID 0042 - Tool disable handler
//!
//! Disables the tool to prevent tightening operations. When disabled, the device
//! will not perform any tightenings regardless of trigger signals.

use crate::handler::data::command_accepted::CommandAccepted;
use crate::handler::{HandlerError, MidHandler};
use crate::observable_state::ObservableState;
use crate::protocol::{Message, Response};

/// MID 0042 - Tool disable
/// Disables the tool to prevent tightening operations
pub struct ToolDisableHandler {
    state: ObservableState,
}

impl ToolDisableHandler {
    pub fn new(state: ObservableState) -> Self {
        Self { state }
    }
}

impl MidHandler for ToolDisableHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0042: Tool disable request");

        // Update device state and broadcast event
        self.state.disable_tool();

        let ack_data = CommandAccepted::with_mid(42);

        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
