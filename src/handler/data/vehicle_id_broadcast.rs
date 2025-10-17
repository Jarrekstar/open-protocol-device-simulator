use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;

/// MID 0052 - Vehicle ID Number (broadcast to subscribers)
///
/// Transmission of identifiers by the controller to the subscriber
/// Revision 1: VIN number only (25 characters)
/// Revision 2: VIN + 3 additional identifiers (4 × 25 characters with parameter IDs)
#[derive(Debug, Clone)]
pub struct VehicleIdBroadcast {
    /// VIN number (25 characters)
    pub vin_number: String,
}

impl VehicleIdBroadcast {
    pub fn new(vin: String) -> Self {
        Self { vin_number: vin }
    }
}

impl ResponseData for VehicleIdBroadcast {
    fn serialize(&self) -> Vec<u8> {
        // Revision 1: VIN number only (no parameter ID)
        // 25 bytes, left-padded with spaces if shorter, truncated if longer
        let vin = if self.vin_number.len() >= 25 {
            self.vin_number[..25].to_string()
        } else {
            format!("{:<25}", self.vin_number)
        };

        FieldBuilder::new()
            .add_str(None, &vin, 25)
            .build()
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
        let broadcast = VehicleIdBroadcast::new("THIS_IS_A_VERY_LONG_VIN_NUMBER_THAT_EXCEEDS_25_CHARS".to_string());
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
}
