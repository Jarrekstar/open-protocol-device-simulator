use super::Response;

const HEADER_SIZE: usize = 20;

/// Serialize a response into Open Protocol format
pub fn serialize_response(response: &Response) -> Vec<u8> {
    // Calculate total length: 20 byte header + data
    let total_length = HEADER_SIZE + response.data.len();

    let mut buffer = Vec::with_capacity(total_length);

    // Length field (4 bytes, zero-padded)
    buffer.extend_from_slice(format!("{:04}", total_length).as_bytes());

    // MID field (4 bytes, zero-padded)
    buffer.extend_from_slice(format!("{:04}", response.mid).as_bytes());

    // Revision field (3 bytes, zero-padded)
    buffer.extend_from_slice(format!("{:03}", response.revision).as_bytes());

    // Reserved/padding (9 bytes of spaces)
    buffer.extend_from_slice(b"         ");

    // Optional data payload
    buffer.extend_from_slice(&response.data);

    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_simple_response() {
        let response = Response {
            mid: 1,
            revision: 1,
            data: Vec::new(),
        };
        let serialized = serialize_response(&response);
        assert_eq!(serialized, b"00200001001         ");
    }

    #[test]
    fn test_serialize_response_with_data() {
        let response = Response {
            mid: 50,
            revision: 1,
            data: b"TEST".to_vec(),
        };
        let serialized = serialize_response(&response);
        assert_eq!(serialized, b"00240050001         TEST");
    }
}
