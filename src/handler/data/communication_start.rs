use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;

/// MID 0002 - Communication start acknowledge
///
/// Response sent after receiving MID 0001 to acknowledge connection
#[derive(Debug, Clone)]
pub struct CommunicationStartAck {
    /// Cell ID (Parameter 01)
    pub cell_id: u32,

    /// Channel ID (Parameter 02)
    pub channel_id: u32,

    /// Controller Name (Parameter 03)
    pub controller_name: String,

    /// Supplier Code (Parameter 04) - Optional
    pub supplier_code: Option<String>,
}

impl CommunicationStartAck {
    /// Create a new communication start acknowledge with default values
    pub fn new() -> Self {
        Self {
            cell_id: 1,
            channel_id: 1,
            controller_name: "Simulator".to_string(),
            supplier_code: Some("SIM".to_string()),
        }
    }

    /// Create with custom values
    pub fn with_values(
        cell_id: u32,
        channel_id: u32,
        controller_name: String,
        supplier_code: Option<String>,
    ) -> Self {
        Self {
            cell_id,
            channel_id,
            controller_name,
            supplier_code,
        }
    }
}

impl Default for CommunicationStartAck {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseData for CommunicationStartAck {
    fn serialize(&self) -> Vec<u8> {
        let mut builder = FieldBuilder::new()
            .add_int(Some(1), self.cell_id as i32, 4)
            .add_int(Some(2), self.channel_id as i32, 2)
            .add_str(Some(3), &self.controller_name, 25);

        if let Some(ref supplier) = self.supplier_code {
            builder = builder.add_str(Some(4), supplier, 3);
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_communication_start_ack_serialization() {
        let ack = CommunicationStartAck::new();
        let data = ack.serialize();

        // Should contain parameters 01, 02, 03, and 04
        assert!(!data.is_empty());
    }

    #[test]
    fn test_custom_values() {
        let ack = CommunicationStartAck::with_values(
            100,
            5,
            "TestController".to_string(),
            Some("TST".to_string()),
        );
        let data = ack.serialize();

        assert!(!data.is_empty());
    }
}
