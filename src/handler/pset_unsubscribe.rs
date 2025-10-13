use crate::handler::{HandlerError, MidHandler};
use crate::handler::data::CommandAccepted;
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};

/// MID 0016 - Unsubscribe from pset selection
/// Responds with MID 0005 (Command accepted)
pub struct PsetUnsubscribeHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl PsetUnsubscribeHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for PsetUnsubscribeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0016: Pset selection unsubscribe request");

        // Update state to mark subscription as inactive
        {
            let mut state = self.state.write().unwrap();
            state.pset_subscribed = false;
        }

        let ack_data = CommandAccepted::with_mid(16);

        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
