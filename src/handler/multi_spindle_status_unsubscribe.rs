use crate::handler::data::CommandAccepted;
use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0092 - Multi-spindle status unsubscribe
/// Client requests to stop receiving multi-spindle status updates
pub struct MultiSpindleStatusUnsubscribeHandler;

impl MidHandler for MultiSpindleStatusUnsubscribeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0092: Multi-spindle status unsubscribe request");

        // Acknowledge unsubscription
        let ack_data = CommandAccepted::with_mid(92);
        Ok(Response::from_data(5, message.revision, ack_data))
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
            mid: 92,
            revision: 1,
            data: vec![],
        };

        let response = handler.handle(&message).unwrap();
        assert_eq!(response.mid, 5); // Command accepted
    }
}
