use crate::handler::{HandlerError, MidHandler};
use crate::handler::data::CommandAccepted;
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};

/// MID 0014 - Subscribe to pset selection
/// Responds with MID 0005 (Command accepted)
pub struct PsetSubscriptionHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl PsetSubscriptionHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for PsetSubscriptionHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0014: Pset selection subscription request");

        // Update state to mark subscription as active
        {
            let mut state = self.state.write().unwrap();
            state.pset_subscribed = true;
        }

        let ack_data = CommandAccepted::with_mid(14);

        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
