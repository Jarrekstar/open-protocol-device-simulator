//! MID 0018 - Parameter set selection handler
//!
//! Selects a specific parameter set (pset) for tightening operations.
//! Each pset defines torque/angle limits and tightening strategy.

use crate::handler::data::error_response::{ErrorCode, ErrorResponse};
use crate::handler::{HandlerError, MidHandler};
use crate::observable_state::ObservableState;
use crate::protocol::{Message, Response};
use crate::pset::SharedPsetRepository;

/// MID 0018 - Parameter set selection
/// Selects a specific parameter set (pset) for tightening operations
pub struct PsetSelectHandler {
    state: ObservableState,
    psets: SharedPsetRepository,
}

impl PsetSelectHandler {
    pub fn new(state: ObservableState, psets: SharedPsetRepository) -> Self {
        Self { state, psets }
    }
}

impl MidHandler for PsetSelectHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        if message.data.len() != 3 || !message.data.iter().all(u8::is_ascii_digit) {
            return Ok(Response::from_data(
                4,
                message.revision,
                ErrorResponse::new(message.mid, ErrorCode::InvalidData),
            ));
        }
        let pset_id = String::from_utf8_lossy(&message.data)
            .parse::<u32>()
            .unwrap_or(0);
        if self.state.read().operation_mode() == crate::tightening_tracker::OperationMode::Job {
            return Ok(Response::from_data(
                4,
                message.revision,
                ErrorResponse::new(message.mid, ErrorCode::PsetCannotBeSet),
            ));
        }
        let Some(pset) = self.psets.read().unwrap().get_by_id(pset_id) else {
            return Ok(Response::from_data(
                4,
                message.revision,
                ErrorResponse::new(message.mid, ErrorCode::ParameterSetNotFound),
            ));
        };

        println!("MID 0018: Parameter set select - Pset ID: {}", pset_id);

        // Update device state and broadcast event
        self.state.set_pset(pset_id, Some(pset.name));

        // Respond with MID 0016 (Command accepted)
        Ok(Response::new(16, message.revision, Vec::new()))
    }
}
