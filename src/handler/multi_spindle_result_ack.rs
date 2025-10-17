use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0102 - Multi-spindle result acknowledge
/// Client acknowledges receipt of multi-spindle result broadcast (MID 0101)
/// No response is sent back for this acknowledgment
pub struct MultiSpindleResultAckHandler;

impl MidHandler for MultiSpindleResultAckHandler {
    fn handle(&self, _message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0102: Multi-spindle result acknowledged by client");

        // No response data required for acknowledgments
        Ok(Response::new(5, 1, Vec::new()))
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

        let response = handler.handle(&message).unwrap();
        assert_eq!(response.mid, 5); // Command accepted (empty response)
    }
}
