use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;

/// MID 0061 - Last tightening result data
///
/// Contains detailed information about a completed tightening operation
#[derive(Debug, Clone)]
pub struct TighteningResult {
    /// Cell ID (Parameter 01)
    pub cell_id: u32,

    /// Channel ID (Parameter 02)
    pub channel_id: u32,

    /// Torque Controller Name (Parameter 03)
    pub controller_name: String,

    /// VIN Number (Parameter 04) - Optional
    pub vin_number: Option<String>,

    /// Job ID (Parameter 05)
    pub job_id: u32,

    /// Parameter Set ID (Parameter 06)
    pub pset_id: u32,

    /// Batch Size (Parameter 07)
    pub batch_size: u32,

    /// Batch Counter (Parameter 08)
    pub batch_counter: u32,

    /// Tightening Status (Parameter 09) - OK=1, NOK=0
    pub tightening_status: bool,

    /// Torque Status (Parameter 10) - OK=1, NOK=0
    pub torque_status: bool,

    /// Angle Status (Parameter 11) - OK=1, NOK=0
    pub angle_status: bool,

    /// Torque Min Limit (Parameter 12) - in Nm
    pub torque_min: f64,

    /// Torque Max Limit (Parameter 13) - in Nm
    pub torque_max: f64,

    /// Torque Final Target (Parameter 14) - in Nm
    pub torque_target: f64,

    /// Torque (Parameter 15) - actual torque in Nm
    pub torque: f64,

    /// Angle Min (Parameter 16) - in degrees
    pub angle_min: f64,

    /// Angle Max (Parameter 17) - in degrees
    pub angle_max: f64,

    /// Angle Final Target (Parameter 18) - in degrees
    pub angle_target: f64,

    /// Angle (Parameter 19) - actual angle in degrees
    pub angle: f64,

    /// Timestamp (Parameter 20) - format: YYYY-MM-DD:HH:MM:SS
    pub timestamp: String,

    /// Last Change in Parameter Set (Parameter 21) - format: YYYY-MM-DD:HH:MM:SS
    pub last_pset_change: Option<String>,

    /// Batch Status (Parameter 22) - OK=1, NOK=0
    pub batch_status: Option<bool>,

    /// Tightening ID (Parameter 23)
    pub tightening_id: Option<u32>,
}

impl TighteningResult {
    /// Create a new tightening result with example values
    #[allow(dead_code)]
    pub fn example() -> Self {
        Self {
            cell_id: 1,
            channel_id: 1,
            controller_name: "Simulator".to_string(),
            vin_number: Some("TEST123456789".to_string()),
            job_id: 1,
            pset_id: 1,
            batch_size: 10,
            batch_counter: 5,
            tightening_status: true,
            torque_status: true,
            angle_status: true,
            torque_min: 10.0,
            torque_max: 15.0,
            torque_target: 12.5,
            torque: 12.3,
            angle_min: 30.0,
            angle_max: 50.0,
            angle_target: 40.0,
            angle: 39.5,
            timestamp: "2025-01-15:10:30:45".to_string(),
            last_pset_change: Some("2025-01-15:09:00:00".to_string()),
            batch_status: Some(true),
            tightening_id: Some(12345),
        }
    }
}

impl ResponseData for TighteningResult {
    fn serialize(&self) -> Vec<u8> {
        // Always send VIN (param 04) - use empty string (25 spaces) if None
        let vin = self.vin_number.as_deref().unwrap_or("");

        // Always send last pset change (param 21) - use empty string (19 spaces) if None
        let pset_change = self.last_pset_change.as_deref().unwrap_or("");

        // Always send batch status (param 22) - 0=NOK, 1=OK, 2=not used
        let batch_status_val = match self.batch_status {
            Some(true) => 1,
            Some(false) => 0,
            None => 2,
        };

        // Always send tightening ID (param 23) - use 0 if None
        let tightening_id = self.tightening_id.unwrap_or(0);

        FieldBuilder::new()
            .add_int(Some(1), self.cell_id as i32, 4)
            .add_int(Some(2), self.channel_id as i32, 2)
            .add_str(Some(3), &self.controller_name, 25)
            .add_str(Some(4), vin, 25)
            .add_int(Some(5), self.job_id as i32, 2)
            .add_int(Some(6), self.pset_id as i32, 3)
            .add_int(Some(7), self.batch_size as i32, 4)
            .add_int(Some(8), self.batch_counter as i32, 4)
            .add_int(Some(9), if self.tightening_status { 1 } else { 0 }, 1)
            .add_int(Some(10), if self.torque_status { 1 } else { 0 }, 1)
            .add_int(Some(11), if self.angle_status { 1 } else { 0 }, 1)
            .add_int(Some(12), (self.torque_min * 100.0) as i32, 6)
            .add_int(Some(13), (self.torque_max * 100.0) as i32, 6)
            .add_int(Some(14), (self.torque_target * 100.0) as i32, 6)
            .add_int(Some(15), (self.torque * 100.0) as i32, 6)
            .add_int(Some(16), self.angle_min as i32, 5)
            .add_int(Some(17), self.angle_max as i32, 5)
            .add_int(Some(18), self.angle_target as i32, 5)
            .add_int(Some(19), self.angle as i32, 5)
            .add_str(Some(20), &self.timestamp, 19)
            .add_str(Some(21), pset_change, 19)
            .add_int(Some(22), batch_status_val, 1)
            .add_int(Some(23), tightening_id as i32, 10)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tightening_result_serialization() {
        let result = TighteningResult::example();
        let data = result.serialize();

        // Should contain multiple parameters
        assert!(!data.is_empty());
        assert!(data.len() > 100); // Complex structure should be large
    }
}
