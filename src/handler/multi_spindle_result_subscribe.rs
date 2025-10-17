use crate::handler::data::CommandAccepted;
use crate::handler::{HandlerError, MidHandler};
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
}
