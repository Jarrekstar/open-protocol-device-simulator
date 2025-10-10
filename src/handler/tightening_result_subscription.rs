use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0060 - Last tightening result data subscribe
/// Responds with MID 0005 (Command accepted)
pub struct TighteningResultSubscriptionHandler;

impl MidHandler for TighteningResultSubscriptionHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0060: Last tightening result subscription request");

        // Respond with MID 0005 (Command accepted)
        Ok(Response::new(5, message.revision, Vec::new()))
    }
}
