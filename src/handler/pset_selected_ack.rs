use crate::handler::{HandlerContext, HandlerError, HandlerResult, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0016 - Parameter set selected acknowledge.
///
/// The client sends this after receiving a MID 0015 broadcast. Acknowledgements
/// do not produce another protocol response.
pub struct PsetSelectedAckHandler;

impl MidHandler for PsetSelectedAckHandler {
    fn handle(&self, _message: &Message) -> Result<Response, HandlerError> {
        Err(HandlerError::Processing(
            "MID 0016 does not produce a response".to_string(),
        ))
    }

    fn handle_with_context(
        &self,
        _message: &Message,
        _context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        println!("MID 0016: Parameter set selected acknowledged by client");
        Ok(HandlerResult::NoResponse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subscriptions::Subscriptions;

    #[test]
    fn acknowledgement_produces_no_response() {
        let handler = PsetSelectedAckHandler;
        let message = Message {
            length: 20,
            mid: 16,
            revision: 1,
            data: Vec::new(),
        };
        let mut subscriptions = Subscriptions::new();
        let mut context = HandlerContext::new(&mut subscriptions);

        assert!(matches!(
            handler.handle_with_context(&message, &mut context),
            Ok(HandlerResult::NoResponse)
        ));
    }
}
