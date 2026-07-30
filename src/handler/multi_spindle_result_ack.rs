use crate::handler::{HandlerContext, HandlerError, HandlerResult, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0102 - Multi-spindle result acknowledge
/// Client acknowledges receipt of multi-spindle result broadcast (MID 0101)
/// No response is sent back for this acknowledgment
pub struct MultiSpindleResultAckHandler;

impl MidHandler for MultiSpindleResultAckHandler {
    fn handle(&self, _message: &Message) -> Result<Response, HandlerError> {
        Err(HandlerError::Processing(
            "MID 0102 does not produce a response".to_string(),
        ))
    }

    fn handle_with_context(
        &self,
        _message: &Message,
        _context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        println!("MID 0102: Multi-spindle result acknowledged by client");
        Ok(HandlerResult::NoResponse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_spindle_result_ack() {
        let handler = MultiSpindleResultAckHandler;
        let message = Message {
            length: 20,
            mid: 102,
            revision: 1,
            data: vec![],
        };

        let mut subscriptions = crate::subscriptions::Subscriptions::new();
        let mut context = HandlerContext::new(&mut subscriptions);

        assert!(matches!(
            handler.handle_with_context(&message, &mut context),
            Ok(HandlerResult::NoResponse)
        ));
    }
}
