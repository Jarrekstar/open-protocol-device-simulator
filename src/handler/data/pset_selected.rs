use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;

/// MID 0015 - Parameter Set Selected
///
/// Notification sent when a parameter set is selected
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PsetSelected {
    /// Parameter Set ID that was selected
    pub pset_id: u32,
}

impl PsetSelected {
    #[allow(dead_code)]
    pub fn new(pset_id: u32) -> Self {
        Self { pset_id }
    }
}

impl ResponseData for PsetSelected {
    fn serialize(&self) -> Vec<u8> {
        // Format: Pset ID (3 digits padded with zeros)
        let builder = FieldBuilder::new().add_int(None, self.pset_id as i32, 3);
        builder.build()
    }
}

impl Default for PsetSelected {
    fn default() -> Self {
        Self::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pset_selected_serialization() {
        let pset = PsetSelected::new(5);
        let data = pset.serialize();
        assert_eq!(&data[..], b"005");
    }

    #[test]
    fn test_pset_selected_large_id() {
        let pset = PsetSelected::new(123);
        let data = pset.serialize();
        assert_eq!(&data[..], b"123");
    }
}
