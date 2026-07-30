use crate::handler::data::CommandAccepted;
use crate::handler::data::error_response::{ErrorCode, ErrorResponse};
use crate::handler::{HandlerContext, HandlerError, HandlerResult, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0093 - Multi-spindle status unsubscribe
/// Client requests to stop receiving multi-spindle status updates
pub struct MultiSpindleStatusUnsubscribeHandler;

impl MidHandler for MultiSpindleStatusUnsubscribeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0093: Multi-spindle status unsubscribe request");

        // Acknowledge unsubscription
        let ack_data = CommandAccepted::with_mid(93);
        Ok(Response::from_data(5, message.revision, ack_data))
    }

    fn handle_with_context(
        &self,
        message: &Message,
        context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        if !message.data.is_empty() {
            return Ok(HandlerResult::Response(Response::from_data(
                4,
                message.revision,
                ErrorResponse::new(message.mid, ErrorCode::InvalidData),
            )));
        }
        if !context
            .subscriptions
            .is_subscribed_to_multi_spindle_status()
        {
            return Ok(HandlerResult::Response(Response::from_data(
                4,
                message.revision,
                ErrorResponse::new(
                    message.mid,
                    ErrorCode::MultiSpindleStatusSubscriptionDoesNotExist,
                ),
            )));
        }
        context.subscriptions.unsubscribe_multi_spindle_status();
        self.handle(message).map(HandlerResult::Response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_spindle_status_unsubscribe() {
        let handler = MultiSpindleStatusUnsubscribeHandler;
        let message = Message {
            length: 20,
            mid: 93,
            revision: 1,
            data: vec![],
        };

        let response = handler.handle(&message).unwrap();
        assert_eq!(response.mid, 5); // Command accepted
        assert_eq!(response.data, b"0093");
    }
}
