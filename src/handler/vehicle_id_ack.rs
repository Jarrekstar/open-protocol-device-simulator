use crate::handler::{HandlerContext, HandlerError, HandlerResult, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0053 - Vehicle ID Number acknowledge
/// Client sends this to acknowledge receipt of MID 0052
/// No response is sent back for this acknowledgement
pub struct VehicleIdAckHandler;

impl MidHandler for VehicleIdAckHandler {
    fn handle(&self, _message: &Message) -> Result<Response, HandlerError> {
        Err(HandlerError::Processing(
            "MID 0053 does not produce a response".to_string(),
        ))
    }

    fn handle_with_context(
        &self,
        _message: &Message,
        _context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        println!("MID 0053: Vehicle ID Number acknowledged by client");
        Ok(HandlerResult::NoResponse)
    }
}
