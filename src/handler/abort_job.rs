//! MID 0127 - Abort Job handler
//!
//! Aborts the currently running Job, if there is one. Per Open Protocol
//! R2.8.0 §5.15.8 the command always answers MID 0005 "Command accepted" and
//! no error reply is defined - even when no Job is running. §3.7.2 lists Abort
//! Job under Pset-style production control too, so it is accepted in every
//! operation mode (it is intentionally not gated in `mode_rejection`).

use crate::handler::data::command_accepted::CommandAccepted;
use crate::handler::{HandlerError, MidHandler};
use crate::observable_state::ObservableState;
use crate::protocol::{Message, Response};

/// MID 0127 - Abort Job
pub struct AbortJobHandler {
    state: ObservableState,
}

impl AbortJobHandler {
    pub fn new(state: ObservableState) -> Self {
        Self { state }
    }
}

impl MidHandler for AbortJobHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        if self.state.abort_job() {
            println!("MID 0127: Abort Job - running Job stopped");
        } else {
            println!("MID 0127: Abort Job - no Job running");
        }

        // Always acknowledge with MID 0005 (Command accepted); the spec defines
        // no error reply for Abort Job.
        Ok(Response::from_data(
            5,
            message.revision,
            CommandAccepted::with_mid(127),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::SimulatorEvent;
    use crate::job::{Job, JobStep};
    use crate::state::DeviceState;
    use crate::tightening_tracker::OperationMode;
    use tokio::sync::broadcast;

    fn abort_message() -> Message {
        Message {
            length: 20,
            mid: 127,
            revision: 1,
            data: vec![],
        }
    }

    fn running_job() -> Job {
        Job {
            id: 7,
            name: "Abortable".to_string(),
            forced_order: 1,
            first_tightening_timeout: 0,
            job_timeout: 0,
            batch_count_mode: 0,
            lock_at_job_done: false,
            use_line_control: false,
            repeat_job: false,
            loosening_mode: 0,
            repair_mode: 0,
            steps: vec![JobStep {
                channel_id: 1,
                pset_id: 1,
                auto_value: true,
                batch_size: 3,
            }],
        }
    }

    #[test]
    fn aborts_running_job_and_acknowledges() {
        let state = DeviceState::new_shared();
        state.write().unwrap().set_job_mode();
        let (tx, mut rx) = broadcast::channel(16);
        let observable = ObservableState::new(state, tx);
        observable
            .select_job(running_job(), Some("Light Duty".to_string()))
            .unwrap();
        assert!(observable.read().is_job_running());
        // Drain the selection events so the assertions below only see the abort.
        while rx.try_recv().is_ok() {}

        let handler = AbortJobHandler::new(observable.clone());
        let response = handler.handle(&abort_message()).unwrap();

        assert_eq!(response.mid, 5);
        assert_eq!(response.data, b"0127");

        let device = observable.read();
        assert!(!device.is_job_mode());
        assert!(!device.is_job_running());
        assert_eq!(device.current_job_id, None);
        assert_eq!(device.operation_mode(), OperationMode::Pset);
        drop(device);

        let events = std::iter::from_fn(|| rx.try_recv().ok()).collect::<Vec<_>>();
        assert!(
            events.iter().any(
                |event| matches!(event, SimulatorEvent::JobAborted { state } if state.job_id == 7)
            ),
            "abort should broadcast JobAborted, got {events:?}"
        );
        assert!(
            events.iter().any(|event| matches!(
                event,
                SimulatorEvent::OperationModeChanged {
                    mode: OperationMode::Pset
                }
            )),
            "abort should broadcast the Job -> Pset transition, got {events:?}"
        );
    }

    #[test]
    fn acknowledges_when_no_job_is_running() {
        let state = DeviceState::new_shared();
        let (tx, mut rx) = broadcast::channel(16);
        let observable = ObservableState::new(state, tx);
        assert_eq!(observable.read().operation_mode(), OperationMode::Pset);

        let handler = AbortJobHandler::new(observable.clone());
        let response = handler.handle(&abort_message()).unwrap();

        assert_eq!(response.mid, 5);
        assert_eq!(response.data, b"0127");
        // No Job was running, so the device stays in Pset mode and stays quiet.
        assert_eq!(observable.read().operation_mode(), OperationMode::Pset);
        assert!(rx.try_recv().is_err());
    }
}
