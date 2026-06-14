//! MID 0019 - Batch size handler
//!
//! Sets the batch size of a parameter set at run time. The batch starts
//! counting when the target PSET is the selected one; otherwise the size is
//! stored and applied when that PSET gets selected.

use crate::handler::data::command_accepted::CommandAccepted;
use crate::handler::data::error_response::{ErrorCode, ErrorResponse};
use crate::handler::{HandlerError, MidHandler};
use crate::observable_state::ObservableState;
use crate::protocol::{Message, Response};
use crate::pset::SharedPsetRepository;

/// MID 0019 - Set parameter set batch size
pub struct BatchSizeHandler {
    state: ObservableState,
    psets: SharedPsetRepository,
}

impl BatchSizeHandler {
    pub fn new(state: ObservableState, psets: SharedPsetRepository) -> Self {
        Self { state, psets }
    }
}

impl MidHandler for BatchSizeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        // Revision 1 data is PPPBB: 3-digit PSET ID and 2-digit batch size.
        if message.data.len() != 5 || !message.data.iter().all(u8::is_ascii_digit) {
            println!(
                "MID 0019 rejected: expected 5 ASCII digits (PPPBB), got {} bytes: {:?}",
                message.data.len(),
                String::from_utf8_lossy(&message.data)
            );
            return Ok(error_response(message, ErrorCode::InvalidData));
        }

        let pset_id = std::str::from_utf8(&message.data[..3])
            .expect("validated ASCII digits")
            .parse::<u32>()
            .expect("validated three-digit PSET ID");
        let batch_size = std::str::from_utf8(&message.data[3..])
            .expect("validated ASCII digits")
            .parse::<u32>()
            .expect("validated two-digit batch size");

        if self.psets.read().unwrap().get_by_id(pset_id).is_none() {
            println!("MID 0019 rejected: PSET {pset_id} not present");
            return Ok(error_response(message, ErrorCode::ParameterSetNotFound));
        }

        println!(
            "MID 0019: Set batch size - PSet: {} -  Size: {}",
            pset_id, batch_size
        );

        // Update device state
        self.state.set_pset_batch_size(pset_id, batch_size);

        let ack_data = CommandAccepted::with_mid(19);

        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}

fn error_response(message: &Message, code: ErrorCode) -> Response {
    Response::from_data(4, message.revision, ErrorResponse::new(19, code))
}
