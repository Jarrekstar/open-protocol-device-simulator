/// Trait for MID-specific response data structures
///
/// Implement this trait on structs that represent the data payload
/// of Open Protocol response messages. The trait provides automatic
/// serialization to the protocol format.
pub trait ResponseData {
    /// Serialize this data structure into Open Protocol format
    ///
    /// Returns the byte representation of the data section (after the 20-byte header)
    fn serialize(&self) -> Vec<u8>;
}

/// Implement ResponseData for empty responses (no data payload)
impl ResponseData for () {
    fn serialize(&self) -> Vec<u8> {
        Vec::new()
    }
}

/// Implement ResponseData for raw byte vectors (pass-through)
impl ResponseData for Vec<u8> {
    fn serialize(&self) -> Vec<u8> {
        self.clone()
    }
}

/// Implement ResponseData for byte slices (pass-through)
impl ResponseData for &[u8] {
    fn serialize(&self) -> Vec<u8> {
        self.to_vec()
    }
}
