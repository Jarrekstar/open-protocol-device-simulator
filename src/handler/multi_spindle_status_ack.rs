use crate::handler::{HandlerContext, HandlerError, HandlerResult, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0092 - Multi-spindle status acknowledge
/// Client acknowledges receipt of multi-spindle status broadcast (MID 0091)
pub struct MultiSpindleStatusAckHandler;

impl MidHandler for MultiSpindleStatusAckHandler {
    fn handle(&self, _message: &Message) -> Result<Response, HandlerError> {
        Err(HandlerError::Processing(
            "MID 0092 does not produce a response".to_string(),
        ))
    }

    fn handle_with_context(
        &self,
        _message: &Message,
        _context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        println!("MID 0092: Multi-spindle status acknowledged by client");
        Ok(HandlerResult::NoResponse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_spindle_status_ack() {
        let handler = MultiSpindleStatusAckHandler;
        let message = Message {
            length: 20,
            mid: 92,
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
