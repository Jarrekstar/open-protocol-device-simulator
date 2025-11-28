use crate::handler::data::CommandAccepted;
use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0003 - Communication stop request
/// Responds with MID 0005 (Command accepted)
pub struct CommunicationStopHandler;

impl Default for CommunicationStopHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CommunicationStopHandler {
    pub fn new() -> Self {
        Self
    }
}

impl MidHandler for CommunicationStopHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0003: Communication stop request");

        // Read device state to populate response
        let ack_data = CommandAccepted::with_mid(3);

        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
