use crate::handler::data::CommandAccepted;
use crate::handler::data::error_response::{ErrorCode, ErrorResponse};
use crate::handler::{HandlerContext, HandlerError, HandlerResult, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0100 - Multi-spindle result subscribe
/// Client requests subscription to multi-spindle tightening results
/// Revision 1 contains no data
pub struct MultiSpindleResultSubscribeHandler;

impl MidHandler for MultiSpindleResultSubscribeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0100: Multi-spindle result subscription request");

        // Acknowledge subscription
        let ack_data = CommandAccepted::with_mid(100);
        Ok(Response::from_data(5, message.revision, ack_data))
    }

    fn handle_with_context(
        &self,
        message: &Message,
        context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        let valid = match message.revision {
            1 => message.data.is_empty(),
            2 => message.data.len() == 10 && message.data.iter().all(u8::is_ascii_digit),
            3..=5 => message.data.len() == 11 && message.data.iter().all(u8::is_ascii_digit),
            _ => false,
        };
        if !valid {
            return Ok(HandlerResult::Response(Response::from_data(
                4,
                message.revision,
                ErrorResponse::new(message.mid, ErrorCode::InvalidData),
            )));
        }
        if context
            .subscriptions
            .is_subscribed_to_multi_spindle_result()
        {
            return Ok(HandlerResult::Response(Response::from_data(
                4,
                message.revision,
                ErrorResponse::new(
                    message.mid,
                    ErrorCode::MultiSpindleResultSubscriptionAlreadyExists,
                ),
            )));
        }
        context
            .subscriptions
            .subscribe_multi_spindle_result_revision(message.revision);
        self.handle(message).map(HandlerResult::Response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_spindle_result_subscribe() {
        let handler = MultiSpindleResultSubscribeHandler;
        let message = Message {
            length: 20,
            mid: 100,
            revision: 1,
            data: vec![],
        };

        let response = handler.handle(&message).unwrap();
        assert_eq!(response.mid, 5); // Command accepted
    }

    #[test]
    fn validates_revision_specific_subscription_data() {
        let handler = MultiSpindleResultSubscribeHandler;
        let mut subscriptions = crate::subscriptions::Subscriptions::new();
        let mut context = HandlerContext::new(&mut subscriptions);
        let accepted = handler
            .handle_with_context(
                &Message {
                    length: 31,
                    mid: 100,
                    revision: 5,
                    data: b"00000000001".to_vec(),
                },
                &mut context,
            )
            .unwrap();
        assert!(matches!(accepted, HandlerResult::Response(response) if response.mid == 5));

        let rejected = handler
            .handle_with_context(
                &Message {
                    length: 20,
                    mid: 100,
                    revision: 5,
                    data: Vec::new(),
                },
                &mut context,
            )
            .unwrap();
        assert!(matches!(rejected, HandlerResult::Response(response) if response.mid == 4));
    }
}
