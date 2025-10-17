use crate::handler::data::CommandAccepted;
use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0090 - Multi-spindle status subscribe
/// Client requests subscription to multi-spindle status updates
pub struct MultiSpindleStatusSubscribeHandler;

impl MidHandler for MultiSpindleStatusSubscribeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0090: Multi-spindle status subscription request");

        // Acknowledge subscription
        let ack_data = CommandAccepted::with_mid(90);
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_spindle_status_subscribe() {
        let handler = MultiSpindleStatusSubscribeHandler;
        let message = Message {
            length: 20,
            mid: 90,
            revision: 1,
            data: vec![],
        };

        let response = handler.handle(&message).unwrap();
        assert_eq!(response.mid, 5); // Command accepted
    }
}
