use crate::handler::{HandlerError, MidHandler};
use crate::handler::data::CommandAccepted;
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};

/// MID 0063 - Last tightening result data unsubscribe
/// Responds with MID 0005 (Command accepted)
pub struct TighteningResultUnsubscribeHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl TighteningResultUnsubscribeHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for TighteningResultUnsubscribeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0063: Last tightening result unsubscribe request");

        // Update state to mark subscription as inactive
        {
            let mut state = self.state.write().unwrap();
            state.tightening_result_subscribed = false;
        }

        let ack_data = CommandAccepted::with_mid(63);

        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
