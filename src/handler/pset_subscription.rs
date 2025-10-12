use crate::handler::{HandlerError, MidHandler};
use crate::handler::data::command_accepted::CommandAccepted;
use crate::protocol::{Message, Response};

/// MID 0014 - Subscribe to pset selection
/// Responds with MID 0005 (Command accepted)
pub struct PsetSubscriptionHandler;

impl MidHandler for PsetSubscriptionHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0014: Pset selection subscription request");

        let ack_data = CommandAccepted::with_mid(14);


        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
