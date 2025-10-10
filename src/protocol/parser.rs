use super::{Message, ProtocolError};
use std::str;

const HEADER_SIZE: usize = 20;

/// Parse a raw Open Protocol message
pub fn parse_message(data: &[u8]) -> Result<Message, ProtocolError> {
    if data.len() < HEADER_SIZE {
        return Err(ProtocolError::MessageTooShort(data.len()));
    }

    // Parse length (bytes 0-3)
    let length_str = str::from_utf8(&data[0..4])
        .map_err(|_| ProtocolError::InvalidLength("not valid UTF-8".to_string()))?;
    let length = length_str
        .parse::<u32>()
        .map_err(|_| ProtocolError::InvalidLength(length_str.to_string()))?;

    // Verify length matches actual message size
    if data.len() != length as usize {
        return Err(ProtocolError::LengthMismatch {
            expected: length as usize,
            actual: data.len(),
        });
    }

    // Parse MID (bytes 4-7)
    let mid_str = str::from_utf8(&data[4..8])
        .map_err(|_| ProtocolError::InvalidMid("not valid UTF-8".to_string()))?;
    let mid = mid_str
        .parse::<u16>()
        .map_err(|_| ProtocolError::InvalidMid(mid_str.to_string()))?;

    // Parse revision (bytes 8-11)
    let revision_str = str::from_utf8(&data[8..11])
        .map_err(|_| ProtocolError::InvalidRevision("not valid UTF-8".to_string()))?;
    let revision = revision_str
        .parse::<u8>()
        .map_err(|_| ProtocolError::InvalidRevision(revision_str.to_string()))?;

    // Extract optional data payload (bytes 20+)
    let data_payload = if data.len() > HEADER_SIZE {
        data[HEADER_SIZE..].to_vec()
    } else {
        Vec::new()
    };

    Ok(Message {
        length,
        mid,
        revision,
        data: data_payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_message() {
        let raw = b"00200001001         ";
        let msg = parse_message(raw).unwrap();
        assert_eq!(msg.length, 20);
        assert_eq!(msg.mid, 1);
        assert_eq!(msg.revision, 1);
        assert_eq!(msg.data.len(), 0);
    }

    #[test]
    fn test_parse_message_with_data() {
        let raw = b"00450050001         SSC044207                ";
        let msg = parse_message(raw).unwrap();
        assert_eq!(msg.length, 45);
        assert_eq!(msg.mid, 50);
        assert_eq!(msg.revision, 1);
        assert_eq!(msg.data.len(), 25);
    }

    #[test]
    fn test_parse_too_short() {
        let raw = b"001";
        assert!(matches!(
            parse_message(raw),
            Err(ProtocolError::MessageTooShort(_))
        ));
    }
}
