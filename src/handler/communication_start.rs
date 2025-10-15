//! MID 0001 - Communication start handler
//!
//! Handles the initial handshake between the integrator and the controller.
//! This is typically the first message sent after TCP connection is established.

use crate::handler::data::CommunicationStartAck;
use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::sync::{Arc, RwLock};

/// MID 0001 - Communication start request
/// Responds with MID 0002 (Communication start acknowledge)
pub struct CommunicationStartHandler {
    state: Arc<RwLock<DeviceState>>,
}

impl CommunicationStartHandler {
    pub fn new(state: Arc<RwLock<DeviceState>>) -> Self {
        Self { state }
    }
}

impl MidHandler for CommunicationStartHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0001: Communication start request");

        // Read device state to populate response
        let ack_data = {
            let state = self.state.read().unwrap();
            CommunicationStartAck::with_values(
                state.cell_id,
                state.channel_id,
                state.controller_name.clone(),
                Some(state.supplier_code.clone()),
            )
        };

        // Respond with MID 0002 (Communication start acknowledge)
        Ok(Response::from_data(2, message.revision, ack_data))
    }
}
