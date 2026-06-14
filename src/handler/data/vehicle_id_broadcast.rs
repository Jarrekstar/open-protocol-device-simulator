use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;
use crate::protocol::revision::{MidRevision, ProtocolSampleData};

/// MID 0052 - Vehicle ID Number (broadcast to subscribers)
///
/// Transmission of identifiers by the controller to the subscriber
/// Revision 1: VIN number only (25 characters)
/// Revision 2: VIN + 3 additional identifiers (4 × 25 characters with parameter IDs)
#[derive(Debug, Clone)]
pub struct VehicleIdBroadcast {
    /// VIN number (25 characters)
    pub vin_number: String,
    pub identifier_part_2: String,
    pub identifier_part_3: String,
    pub identifier_part_4: String,
}

impl VehicleIdBroadcast {
    pub fn new(vin: String) -> Self {
        Self {
            vin_number: vin,
            identifier_part_2: String::new(),
            identifier_part_3: String::new(),
            identifier_part_4: String::new(),
        }
    }

    pub fn with_samples(vin: String, samples: &ProtocolSampleData) -> Self {
        Self {
            vin_number: vin,
            identifier_part_2: samples.identifier_part_2.clone(),
            identifier_part_3: samples.identifier_part_3.clone(),
            identifier_part_4: samples.identifier_part_4.clone(),
        }
    }

    pub fn serialize_revision(&self, revision: MidRevision) -> Vec<u8> {
        if revision == 1 {
            return FieldBuilder::new()
                .add_str(None, &self.vin_number, 25)
                .build();
        }

        FieldBuilder::new()
            .add_str(Some(1), &self.vin_number, 25)
            .add_str(Some(2), &self.identifier_part_2, 25)
            .add_str(Some(3), &self.identifier_part_3, 25)
            .add_str(Some(4), &self.identifier_part_4, 25)
            .build()
    }
}

impl ResponseData for VehicleIdBroadcast {
    fn serialize(&self) -> Vec<u8> {
        self.serialize_revision(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_id_broadcast_exact_length() {
        let broadcast = VehicleIdBroadcast::new("SSC044207                ".to_string());
        let data = broadcast.serialize();
        assert_eq!(data.len(), 25);
        assert_eq!(&data[..], b"SSC044207                ");
    }

    #[test]
    fn test_vehicle_id_broadcast_short_vin() {
        let broadcast = VehicleIdBroadcast::new("TEST123".to_string());
        let data = broadcast.serialize();
        assert_eq!(data.len(), 25);
        assert_eq!(&data[..], b"TEST123                  ");
    }

    #[test]
    fn test_vehicle_id_broadcast_long_vin() {
        let broadcast = VehicleIdBroadcast::new(
            "THIS_IS_A_VERY_LONG_VIN_NUMBER_THAT_EXCEEDS_25_CHARS".to_string(),
        );
        let data = broadcast.serialize();
        assert_eq!(data.len(), 25);
        assert_eq!(&data[..], b"THIS_IS_A_VERY_LONG_VIN_N");
    }

    #[test]
    fn test_vehicle_id_broadcast_empty() {
        let broadcast = VehicleIdBroadcast::new(String::new());
        let data = broadcast.serialize();
        assert_eq!(data.len(), 25);
        assert_eq!(&data[..], b"                         ");
    }

    #[test]
    fn revision_two_contains_four_identifier_parts() {
        let samples = ProtocolSampleData::default();
        let broadcast = VehicleIdBroadcast::with_samples("VIN123".to_string(), &samples);
        let data = broadcast.serialize_revision(2);
        assert_eq!(data.len(), 108);
        let text = String::from_utf8(data).unwrap();
        assert!(text.starts_with("01VIN123"));
        assert!(text.contains("02WORKORDER-0001"));
        assert!(text.contains("03MODEL-SIMULATOR"));
        assert!(text.contains("04BODY-00000001"));
    }
}
