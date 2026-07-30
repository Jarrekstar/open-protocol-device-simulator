use crate::handler::data::CommandAccepted;
use crate::handler::data::error_response::{ErrorCode, ErrorResponse};
use crate::handler::{HandlerContext, HandlerError, HandlerResult, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0014 - Subscribe to pset selection
/// Responds with MID 0005 (Command accepted)
///
/// Note: Subscription state is managed per-connection in ConnectionSession.
/// This handler only returns the acknowledgment response.
pub struct PsetSubscriptionHandler;

impl MidHandler for PsetSubscriptionHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0014: Pset selection subscription request");

        let ack_data = CommandAccepted::with_mid(14);

        // Respond with MID 0005 (Command accepted)
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
        if context.subscriptions.is_subscribed_to_pset_selection() {
            return Ok(HandlerResult::Response(Response::from_data(
                4,
                message.revision,
                ErrorResponse::new(
                    message.mid,
                    ErrorCode::PsetSelectionSubscriptionAlreadyExists,
                ),
            )));
        }
        context
            .subscriptions
            .subscribe_pset_selection_revision(message.revision);
        self.handle(message).map(HandlerResult::Response)
    }
}
