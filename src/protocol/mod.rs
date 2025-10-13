pub mod field;
pub mod parser;
pub mod response_data;
pub mod serializer;

use response_data::ResponseData;
use thiserror::Error;

/// Open Protocol message structure
/// Header: 20 bytes (length + MID + revision + reserved)
/// Data: Optional MID-specific payload
#[derive(Debug, Clone)]
pub struct Message {
    #[allow(dead_code)]
    pub length: u32,      // Total message length (bytes 0-3)
    pub mid: u16,         // Message ID (bytes 4-7)
    pub revision: u8,     // Protocol revision (bytes 8-10)
    pub data: Vec<u8>,    // Optional MID-specific data (bytes 20+)
}

/// Response message to be sent back
#[derive(Debug, Clone)]
pub struct Response {
    pub mid: u16,
    pub revision: u8,
    pub data: Vec<u8>,
}

impl Response {
    /// Create a new response with raw data
    pub fn new(mid: u16, revision: u8, data: Vec<u8>) -> Self {
        Self { mid, revision, data }
    }

    /// Create a response from a type that implements ResponseData
    pub fn from_data(mid: u16, revision: u8, data: impl ResponseData) -> Self {
        Self {
            mid,
            revision,
            data: data.serialize(),
        }
    }
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Message too short: expected at least 20 bytes, got {0}")]
    MessageTooShort(usize),

    #[error("Invalid length field: {0}")]
    InvalidLength(String),

    #[error("Invalid MID field: {0}")]
    InvalidMid(String),

    #[error("Invalid revision field: {0}")]
    InvalidRevision(String),

    #[error("Length mismatch: header says {expected}, actual message is {actual}")]
    LengthMismatch { expected: usize, actual: usize },
}
