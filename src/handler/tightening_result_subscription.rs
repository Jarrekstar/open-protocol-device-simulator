use crate::handler::{HandlerError, MidHandler};
use crate::handler::data::command_accepted::CommandAccepted;
use crate::protocol::{Message, Response};

/// MID 0060 - Last tightening result data subscribe
/// Responds with MID 0005 (Command accepted)
pub struct TighteningResultSubscriptionHandler;

impl MidHandler for TighteningResultSubscriptionHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0060: Last tightening result subscription request");

        let ack_data = CommandAccepted::with_mid(60);


        // Respond with MID 0005 (Command accepted)
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}
