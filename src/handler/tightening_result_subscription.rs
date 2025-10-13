use crate::handler::{HandlerError, MidHandler};
use crate::handler::data::CommandAccepted;
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};

/// MID 0060 - Last tightening result data subscribe
/// Responds with MID 0005 (Command accepted)
pub struct TighteningResultSubscriptionHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl TighteningResultSubscriptionHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for TighteningResultSubscriptionHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0060: Last tightening result subscription request");

        // Update state to mark subscription as active
        {
            let mut state = self.state.write().unwrap();
            state.tightening_result_subscribed = true;
        }

        let ack_data = CommandAccepted::with_mid(60);

        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
