//! MID 0020 - Reset parameter set batch counter
//!
//! Resets the batch counter of the running parameter set at runtime.
//! The batch size remains unchanged, only the counter is reset to 0.

use crate::handler::data::command_accepted::CommandAccepted;
use crate::handler::data::error_response::{ErrorCode, ErrorResponse};
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
        // Revision 1 data is the PSET ID: exactly 3 ASCII digits.
        if message.data.len() != 3 || !message.data.iter().all(u8::is_ascii_digit) {
            println!(
                "MID 0020 rejected: expected 3 ASCII digits (PPP), got {} bytes: {:?}",
                message.data.len(),
                String::from_utf8_lossy(&message.data)
            );
            return Ok(Response::from_data(
                4,
                message.revision,
                ErrorResponse::new(20, ErrorCode::InvalidData),
            ));
        }
        let pset_id = std::str::from_utf8(&message.data)
            .expect("validated ASCII digits")
            .parse::<u32>()
            .expect("validated three-digit PSET ID");

        let was_batch_running = {
            let mut state = self.state.write().unwrap();
            // Only the running (selected) PSET's batch counter can be reset.
            state.current_pset_id == Some(pset_id) && state.reset_batch()
        };

        if was_batch_running {
            println!(
                "MID 0020: Reset batch counter for pset {} - counter reset to 0",
                pset_id
            );
            let ack_data = CommandAccepted::with_mid(20);
            Ok(Response::from_data(5, message.revision, ack_data))
        } else {
            // Not the running PSET or no batch configured:
            // error 04 "Parameter set not running"
            println!(
                "MID 0020: Reset batch counter failed - pset {} not running",
                pset_id
            );
            let error_data = ErrorResponse::new(20, ErrorCode::ParameterSetNotRunning);
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
