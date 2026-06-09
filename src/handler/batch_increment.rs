//! MID 0128 - Job batch increment handler
//!
//! Increments the batch counter without a tightening result.
//! Used by integrators to skip a bolt position (e.g., after max retries).

use crate::handler::data::command_accepted::CommandAccepted;
use crate::handler::{HandlerError, MidHandler};
use crate::observable_state::ObservableState;
use crate::protocol::{Message, Response};
use crate::pset::SharedPsetRepository;

/// MID 0128 - Job batch increment
/// Increments the batch counter to skip a bolt position
pub struct BatchIncrementHandler {
    state: ObservableState,
    psets: SharedPsetRepository,
}

impl BatchIncrementHandler {
    pub fn new(state: ObservableState, psets: SharedPsetRepository) -> Self {
        Self { state, psets }
    }
}

impl MidHandler for BatchIncrementHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        let (new_counter, target_size, progress, runtime, tool_disabled) = {
            let mut state = self.state.write();
            let tool_was_enabled = state.tool_enabled;
            let (new_counter, progress) = state.increment_batch_with_progress();
            let target_size = state.tightening_tracker.batch_size();
            let runtime = state.job_runtime_state();
            (
                new_counter,
                target_size,
                progress,
                runtime,
                tool_was_enabled && !state.tool_enabled,
            )
        };

        println!(
            "MID 0128: Job batch increment - new counter: {}",
            new_counter
        );

        // Broadcast progress update to frontend
        self.state
            .broadcast_auto_progress(new_counter, target_size, true);
        if let (Some(progress), Some(runtime)) = (progress, runtime) {
            if progress.step_changed {
                let pset_name = self
                    .psets
                    .read()
                    .unwrap()
                    .get_by_id(runtime.current_pset_id)
                    .map(|pset| pset.name)
                    .unwrap_or_else(|| "Unknown".to_string());
                {
                    let mut state = self.state.write();
                    state.current_pset_name = Some(pset_name.clone());
                }
                self.state
                    .broadcast(crate::events::SimulatorEvent::PsetChanged {
                        pset_id: runtime.current_pset_id,
                        pset_name,
                    });
                self.state
                    .broadcast(crate::events::SimulatorEvent::JobStepChanged {
                        state: runtime.clone(),
                        previous_step: progress.previous_step_index as u32 + 1,
                    });
            }
            self.state
                .broadcast(crate::events::SimulatorEvent::JobProgress {
                    state: runtime.clone(),
                });
            if let Some(status) = progress.completed_status {
                let mut completed = runtime;
                completed.status = status;
                completed.total_progress = completed.total_batch_size;
                self.state
                    .broadcast(crate::events::SimulatorEvent::JobCompleted { state: completed });
            }
        }
        if tool_disabled {
            self.state
                .broadcast(crate::events::SimulatorEvent::ToolStateChanged { enabled: false });
        }

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

        let handler = BatchIncrementHandler::new(
            observable.clone(),
            crate::pset::create_default_repository(),
        );

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
