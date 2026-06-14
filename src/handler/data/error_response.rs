use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;

/// MID 0004 - Error/NAK Response
///
/// Negative acknowledgment sent when a request fails
#[derive(Debug, Clone)]
pub struct ErrorResponse {
    /// The MID that caused the error
    pub failed_mid: u16,

    /// Error code
    pub error_code: ErrorCode,
}

/// Open Protocol error codes (R2.8.0 Table 18)
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum ErrorCode {
    /// Invalid data
    InvalidData = 1,
    /// Parameter set ID not present
    ParameterSetNotFound = 2,
    /// Parameter set can not be set
    PsetCannotBeSet = 3,
    /// Parameter set not running
    ParameterSetNotRunning = 4,
    /// VIN input source not granted
    VehicleIdNotGranted = 8,
    /// Job ID not present
    JobNotFound = 17,
    /// Job info subscription already exists
    SubscriptionAlreadyExists = 18,
    /// Job info subscription does not exist
    SubscriptionDoesNotExist = 19,
    /// Job can not be set
    JobCannotBeSet = 20,
    /// Job not running
    JobNotRunning = 21,
    /// Reject connection, client already connected
    ClientAlreadyConnected = 96,
    /// MID revision unsupported
    MidRevisionUnsupported = 97,
    /// Unknown MID
    GenericError = 99,
}

impl ErrorResponse {
    pub fn new(failed_mid: u16, error_code: ErrorCode) -> Self {
        Self {
            failed_mid,
            error_code,
        }
    }

    /// MID revision unsupported error
    #[allow(dead_code)]
    pub fn revision_unsupported(failed_mid: u16) -> Self {
        Self::new(failed_mid, ErrorCode::MidRevisionUnsupported)
    }

    /// Client already connected error
    #[allow(dead_code)]
    pub fn already_connected(failed_mid: u16) -> Self {
        Self::new(failed_mid, ErrorCode::ClientAlreadyConnected)
    }

    /// Invalid data error
    #[allow(dead_code)]
    pub fn invalid_data(failed_mid: u16) -> Self {
        Self::new(failed_mid, ErrorCode::InvalidData)
    }

    /// Generic error
    pub fn generic(failed_mid: u16) -> Self {
        Self::new(failed_mid, ErrorCode::GenericError)
    }
}

impl ResponseData for ErrorResponse {
    fn serialize(&self) -> Vec<u8> {
        // Format: Failed MID (4 digits) + Error Code (2 digits)
        let builder = FieldBuilder::new()
            .add_int(None, self.failed_mid as i32, 4)
            .add_int(None, self.error_code as i32, 2);

        builder.build()
    }
}

impl Default for ErrorResponse {
    fn default() -> Self {
        Self::new(0, ErrorCode::GenericError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse::new(18, ErrorCode::ParameterSetNotFound);
        let data = error.serialize();

        // Should contain MID (4 chars) + error code (2 chars) = 6 bytes
        assert_eq!(data.len(), 6);
        assert_eq!(&data[..], b"001802");
    }

    #[test]
    fn test_revision_unsupported() {
        let error = ErrorResponse::revision_unsupported(1);
        let data = error.serialize();
        assert_eq!(&data[..], b"000197");
    }
}
