use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;
use crate::protocol::revision::{MidRevision, ProtocolSampleData};

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

    pub samples: ProtocolSampleData,
}

impl CommunicationStartAck {
    /// Create a new communication start acknowledge with default values
    pub fn new() -> Self {
        Self {
            cell_id: 1,
            channel_id: 1,
            controller_name: "Simulator".to_string(),
            supplier_code: Some("SIM".to_string()),
            samples: ProtocolSampleData::default(),
        }
    }

    /// Create with custom values
    pub fn with_values(
        cell_id: u32,
        channel_id: u32,
        controller_name: String,
        supplier_code: Option<String>,
        samples: ProtocolSampleData,
    ) -> Self {
        Self {
            cell_id,
            channel_id,
            controller_name,
            supplier_code,
            samples,
        }
    }

    pub fn serialize_revision(&self, revision: MidRevision) -> Vec<u8> {
        let mut builder = FieldBuilder::new()
            .add_int(Some(1), self.cell_id as i32, 4)
            .add_int(Some(2), self.channel_id as i32, 2)
            .add_str(Some(3), &self.controller_name, 25);

        if revision >= 2 {
            builder = builder.add_str(
                Some(4),
                self.supplier_code.as_deref().unwrap_or_default(),
                3,
            );
        }
        if revision >= 3 {
            builder = builder
                .add_str(Some(5), &self.samples.open_protocol_version, 19)
                .add_str(Some(6), &self.samples.controller_software_version, 19)
                .add_str(Some(7), &self.samples.tool_software_version, 19);
        }
        if revision >= 4 {
            builder = builder
                .add_str(Some(8), &self.samples.rbu_type, 19)
                .add_str(Some(9), &self.samples.controller_serial_number, 25);
        }
        if revision >= 5 {
            builder = builder
                .add_int(Some(10), self.samples.system_type as i32, 1)
                .add_int(Some(11), self.samples.system_subtype as i32, 1);
        }
        if revision >= 6 {
            builder = builder
                .add_int(
                    Some(12),
                    i32::from(self.samples.supports_sequence_number),
                    1,
                )
                .add_int(
                    Some(13),
                    i32::from(self.samples.supports_message_linking),
                    1,
                )
                .add_int(Some(14), self.samples.station_id as i32, 2);
        }

        builder.build()
    }
}

impl Default for CommunicationStartAck {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseData for CommunicationStartAck {
    fn serialize(&self) -> Vec<u8> {
        self.serialize_revision(1)
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
            ProtocolSampleData::default(),
        );
        let data = ack.serialize();

        assert!(!data.is_empty());
    }

    #[test]
    fn revision_six_contains_capability_fields() {
        let samples = ProtocolSampleData {
            supports_sequence_number: true,
            supports_message_linking: true,
            station_id: 7,
            ..ProtocolSampleData::default()
        };
        let ack = CommunicationStartAck::with_values(
            1,
            1,
            "Simulator".to_string(),
            Some("SIM".to_string()),
            samples,
        );
        let data = String::from_utf8(ack.serialize_revision(6)).unwrap();
        assert!(data.contains("121"));
        assert!(data.contains("131"));
        assert!(data.ends_with("1407"));
    }
}
