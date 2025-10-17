use crate::handler::data::CommandAccepted;
use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0103 - Multi-spindle result unsubscribe
/// Client requests to stop receiving multi-spindle result updates
pub struct MultiSpindleResultUnsubscribeHandler;

impl MidHandler for MultiSpindleResultUnsubscribeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0103: Multi-spindle result unsubscribe request");

        // Acknowledge unsubscription
        let ack_data = CommandAccepted::with_mid(103);
        Ok(Response::from_data(5, message.revision, ack_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_spindle_result_unsubscribe() {
        let handler = MultiSpindleResultUnsubscribeHandler;
        let message = Message {
            length: 20,
            mid: 103,
            revision: 1,
            data: vec![],
        };

        let response = handler.handle(&message).unwrap();
        assert_eq!(response.mid, 5); // Command accepted
    }
}
