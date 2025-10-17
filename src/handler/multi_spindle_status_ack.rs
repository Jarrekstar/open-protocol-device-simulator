use crate::handler::{HandlerError, MidHandler};
use crate::protocol::{Message, Response};

/// MID 0093 - Multi-spindle status acknowledge
/// Client acknowledges receipt of multi-spindle status broadcast (MID 0091)
pub struct MultiSpindleStatusAckHandler;

impl MidHandler for MultiSpindleStatusAckHandler {
    fn handle(&self, _message: &Message) -> Result<Response, HandlerError> {
        println!("MID 0093: Multi-spindle status acknowledged by client");

        // No response data required for acknowledgments
        Ok(Response::new(5, 1, Vec::new()))
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
            mid: 93,
            revision: 1,
            data: vec![],
        };

        let response = handler.handle(&message).unwrap();
        assert_eq!(response.mid, 5); // Command accepted (empty response)
    }
}
