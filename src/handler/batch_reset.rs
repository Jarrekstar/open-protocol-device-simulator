//! MID 0020 - Reset parameter set batch counter
//!
//! Resets the batch counter of the running parameter set at runtime.
//! The batch size remains unchanged, only the counter is reset to 0.

use crate::handler::data::command_accepted::CommandAccepted;
use crate::handler::data::error_response::ErrorResponse;
use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};

/// MID 0020 - Reset parameter set batch counter
pub struct BatchResetHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl BatchResetHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for BatchResetHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        // Extract pset ID from message data if present (bytes 0-2, 3 ASCII digits)
        let pset_id = if message.data.len() >= 3 {
            String::from_utf8_lossy(&message.data[0..3])
                .trim()
                .parse::<u32>()
                .unwrap_or(0)
        } else {
            0
        };

        let was_batch_mode = {
            let mut state = self.state.write().unwrap();
            state.reset_batch()
        };

        if was_batch_mode {
            println!(
                "MID 0020: Reset batch counter for pset {} - counter reset to 0",
                pset_id
            );
            let ack_data = CommandAccepted::with_mid(20);
            Ok(Response::from_data(5, message.revision, ack_data))
        } else {
            // Not in batch mode - return error
            println!(
                "MID 0020: Reset batch counter failed - not in batch mode (pset {})",
                pset_id
            );
            let error_data = ErrorResponse::invalid_data(20);
            Ok(Response::from_data(4, message.revision, error_data))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Message;

    #[test]
    fn test_batch_reset_in_batch_mode() {
        let state = DeviceState::new_shared();

        // Enable batch mode and add some tightenings
        {
            let mut s = state.write().unwrap();
            s.set_batch_size(5);
            s.tightening_tracker.add_tightening(true);
            s.tightening_tracker.add_tightening(true);
            assert_eq!(s.tightening_tracker.counter(), 2);
        }

        let handler = BatchResetHandler::new(Arc::clone(&state));

        // Create a MID 0020 message with pset ID "001"
        let message = Message {
            length: 23,
            mid: 20,
            revision: 1,
            data: b"001".to_vec(),
        };

        let response = handler.handle(&message).unwrap();
        assert_eq!(response.mid, 5); // Command accepted

        // Verify counter was reset
        let s = state.read().unwrap();
        assert_eq!(s.tightening_tracker.counter(), 0);
    }

    #[test]
    fn test_batch_reset_not_in_batch_mode() {
        let state = DeviceState::new_shared();

        // Don't enable batch mode (stay in single mode)
        let handler = BatchResetHandler::new(Arc::clone(&state));

        let message = Message {
            length: 23,
            mid: 20,
            revision: 1,
            data: b"001".to_vec(),
        };

        let response = handler.handle(&message).unwrap();
        assert_eq!(response.mid, 4); // Command error
    }
}
