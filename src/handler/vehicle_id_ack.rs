use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0053 - Vehicle ID Number acknowledge
/// Client sends this to acknowledge receipt of MID 0052
/// No response is sent back for this acknowledgement
pub struct VehicleIdAckHandler;

impl MidHandler for VehicleIdAckHandler {
    fn handle(&self, _message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0053: Vehicle ID Number acknowledged by client");

        // This is an acknowledgement message - typically no response is needed
        // However, the current architecture requires a response, so we'll return an empty response
        // In a real implementation, you might want to track ACKs or handle this differently

        // Return a simple response with no data (just the header will be sent)
        Ok(Response::new(5, 1, Vec::new()))
    }
}
