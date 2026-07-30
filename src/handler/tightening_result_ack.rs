use crate::handler::{HandlerContext, HandlerError, HandlerResult, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0062 - Last tightening result data acknowledge
/// Client sends this to acknowledge receipt of MID 0061
/// No response is sent back for this acknowledgement
pub struct TighteningResultAckHandler;

impl MidHandler for TighteningResultAckHandler {
    fn handle(&self, _message: &Message) -> Result<Response, HandlerError> {
        Err(HandlerError::Processing(
            "MID 0062 does not produce a response".to_string(),
        ))
    }

    fn handle_with_context(
        &self,
        _message: &Message,
        _context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        println!("MID 0062: Last tightening result data acknowledged by client");
        Ok(HandlerResult::NoResponse)
    }
}
