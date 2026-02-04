//! MID 0128 - Job batch increment handler
//!
//! Increments the batch counter without a tightening result.
//! Used by integrators to skip a bolt position (e.g., after max retries).

use crate::handler::data::command_accepted::CommandAccepted;
use crate::handler::{HandlerError, MidHandler};
use crate::observable_state::ObservableState;
use crate::protocol::{Message, Response};

/// MID 0128 - Job batch increment
/// Increments the batch counter to skip a bolt position
pub struct BatchIncrementHandler {
    state: ObservableState,
}

impl BatchIncrementHandler {
    pub fn new(state: ObservableState) -> Self {
        Self { state }
    }
}

impl MidHandler for BatchIncrementHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        let (new_counter, target_size) = {
            let mut state = self.state.write();
            let new_counter = state.increment_batch();
            let target_size = state.tightening_tracker.batch_size();
            (new_counter, target_size)
        };

        println!(
            "MID 0128: Job batch increment - new counter: {}",
            new_counter
        );

        // Broadcast progress update to frontend
        self.state
            .broadcast_auto_progress(new_counter, target_size, true);

        let ack_data = CommandAccepted::with_mid(128);

        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Message;
    use crate::state::DeviceState;
    use tokio::sync::broadcast;

    fn create_test_observable() -> ObservableState {
        let state = DeviceState::new_shared();
        let (tx, _) = broadcast::channel(16);
        ObservableState::new(state, tx)
    }

    #[test]
    fn test_batch_increment() {
        let observable = create_test_observable();

        // Enable batch mode first
        {
            let mut s = observable.write();
            s.set_batch_size(5);
        }

        let handler = BatchIncrementHandler::new(observable.clone());

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
        let s = observable.read();
        assert_eq!(s.tightening_tracker.counter(), 1);
    }
}
