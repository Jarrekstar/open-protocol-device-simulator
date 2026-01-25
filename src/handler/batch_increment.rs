//! MID 0128 - Job batch increment handler
//!
//! Increments the batch counter without a tightening result.
//! Used by integrators to skip a bolt position (e.g., after max retries).

use crate::handler::data::command_accepted::CommandAccepted;
use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};

/// MID 0128 - Job batch increment
/// Increments the batch counter to skip a bolt position
pub struct BatchIncrementHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl BatchIncrementHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for BatchIncrementHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        let new_counter = {
            let mut state = self.state.write().unwrap();
            state.increment_batch()
        };

        println!(
            "MID 0128: Job batch increment - new counter: {}",
            new_counter
        );

        let ack_data = CommandAccepted::with_mid(128);

        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Message;

    #[test]
    fn test_batch_increment() {
        let state = DeviceState::new_shared();

        // Enable batch mode first
        {
            let mut s = state.write().unwrap();
            s.set_batch_size(5);
        }

        let handler = BatchIncrementHandler::new(Arc::clone(&state));

        // Create a MID 0128 message
        let message = Message {
            length: 20,
            mid: 128,
            revision: 1,
            data: vec![],
        };

        let response = handler.handle(&message).unwrap();
        assert_eq!(response.mid, 5); // Command accepted

        // Verify counter was incremented
        let s = state.read().unwrap();
        assert_eq!(s.tightening_tracker.counter(), 1);
    }
}
