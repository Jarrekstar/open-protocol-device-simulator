use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};

/// MID 9999 - Keep alive / Heartbeat
/// Responds with MID 9999 (Keep alive acknowledge)
pub struct KeepAliveHandler;

impl MidHandler for KeepAliveHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 9999: Keep alive ping");

        // Respond with MID 9999 (Keep alive acknowledge)
        Ok(Response::new(9999, message.revision, Vec::new()))
    }
}
