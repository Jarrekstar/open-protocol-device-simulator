use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;
use crate::protocol::revision::{MidRevision, ProtocolSampleData};
use serde::{Deserialize, Serialize};

/// MID 0061 - Last tightening result data
///
/// Contains detailed information about a completed tightening operation
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn serialize_revision(
        &self,
        revision: MidRevision,
        samples: &ProtocolSampleData,
    ) -> Vec<u8> {
        match revision {
            1 => self.serialize_revision_1(),
            2..=7 | 998 => self.serialize_extended(revision, samples),
            999 => self.serialize_light(),
            _ => Vec::new(),
        }
    }

    fn batch_status_value(&self) -> u64 {
        match self.batch_status {
            Some(true) => 1,
            Some(false) => 0,
            None => 2,
        }
    }

    fn serialize_revision_1(&self) -> Vec<u8> {
        let vin = self.vin_number.as_deref().unwrap_or("");
        let pset_change = self.last_pset_change.as_deref().unwrap_or("");

        FieldBuilder::new()
            .add_uint(Some(1), self.cell_id as u64, 4)
            .add_uint(Some(2), self.channel_id as u64, 2)
            .add_str(Some(3), &self.controller_name, 25)
            .add_str(Some(4), vin, 25)
            .add_uint(Some(5), self.job_id as u64, 2)
            .add_uint(Some(6), self.pset_id as u64, 3)
            .add_uint(Some(7), self.batch_size as u64, 4)
            .add_uint(Some(8), self.batch_counter as u64, 4)
            .add_uint(Some(9), u64::from(self.tightening_status), 1)
            .add_uint(Some(10), u64::from(self.torque_status), 1)
            .add_uint(Some(11), u64::from(self.angle_status), 1)
            .add_uint(Some(12), (self.torque_min * 100.0) as u64, 6)
            .add_uint(Some(13), (self.torque_max * 100.0) as u64, 6)
            .add_uint(Some(14), (self.torque_target * 100.0) as u64, 6)
            .add_uint(Some(15), (self.torque * 100.0) as u64, 6)
            .add_uint(Some(16), self.angle_min as u64, 5)
            .add_uint(Some(17), self.angle_max as u64, 5)
            .add_uint(Some(18), self.angle_target as u64, 5)
            .add_uint(Some(19), self.angle as u64, 5)
            .add_str(Some(20), &self.timestamp, 19)
            .add_str(Some(21), pset_change, 19)
            .add_uint(Some(22), self.batch_status_value(), 1)
            .add_uint(Some(23), self.tightening_id.unwrap_or(0) as u64, 10)
            .build()
    }

    fn serialize_extended(&self, revision: MidRevision, samples: &ProtocolSampleData) -> Vec<u8> {
        let vin = self.vin_number.as_deref().unwrap_or("");
        let pset_change = self.last_pset_change.as_deref().unwrap_or("");
        let mut data = FieldBuilder::new()
            .add_uint(Some(1), self.cell_id as u64, 4)
            .add_uint(Some(2), self.channel_id as u64, 2)
            .add_str(Some(3), &self.controller_name, 25)
            .add_str(Some(4), vin, 25)
            .add_uint(Some(5), self.job_id as u64, 4)
            .add_uint(Some(6), self.pset_id as u64, 3)
            .add_uint(Some(7), samples.tightening_strategy as u64, 2)
            .add_uint(Some(8), samples.tightening_strategy_options as u64, 5)
            .add_uint(Some(9), self.batch_size as u64, 4)
            .add_uint(Some(10), self.batch_counter as u64, 4)
            .add_uint(Some(11), u64::from(self.tightening_status), 1)
            .add_uint(Some(12), self.batch_status_value(), 1)
            .add_uint(Some(13), u64::from(self.torque_status), 1)
            .add_uint(Some(14), u64::from(self.angle_status), 1)
            .add_uint(Some(15), 1, 1)
            .add_uint(Some(16), 1, 1)
            .add_uint(Some(17), 1, 1)
            .add_uint(Some(18), 1, 1)
            .add_uint(Some(19), 1, 1)
            .add_uint(Some(20), samples.tightening_error_status as u64, 10)
            .add_uint(Some(21), (self.torque_min * 100.0) as u64, 6)
            .add_uint(Some(22), (self.torque_max * 100.0) as u64, 6)
            .add_uint(Some(23), (self.torque_target * 100.0) as u64, 6)
            .add_uint(Some(24), (self.torque * 100.0) as u64, 6)
            .add_uint(Some(25), self.angle_min as u64, 5)
            .add_uint(Some(26), self.angle_max as u64, 5)
            .add_uint(Some(27), self.angle_target as u64, 5)
            .add_uint(Some(28), self.angle as u64, 5)
            .add_uint(Some(29), samples.rundown_angle_min as u64, 5)
            .add_uint(Some(30), samples.rundown_angle_max as u64, 5)
            .add_uint(Some(31), samples.rundown_angle as u64, 5)
            .add_uint(Some(32), samples.current_monitoring_min as u64, 3)
            .add_uint(Some(33), samples.current_monitoring_max as u64, 3)
            .add_uint(Some(34), samples.current_monitoring_value as u64, 3)
            .add_uint(Some(35), samples.self_tap_min as u64, 6)
            .add_uint(Some(36), samples.self_tap_max as u64, 6)
            .add_uint(Some(37), samples.self_tap_torque as u64, 6)
            .add_uint(Some(38), samples.prevail_torque_min as u64, 6)
            .add_uint(Some(39), samples.prevail_torque_max as u64, 6)
            .add_uint(Some(40), samples.prevail_torque as u64, 6)
            .add_uint(Some(41), self.tightening_id.unwrap_or(0) as u64, 10)
            .add_uint(Some(42), samples.job_sequence_number as u64, 5)
            .add_uint(Some(43), 0, 5)
            .add_str(Some(44), &samples.tool_serial_number, 14)
            .add_str(Some(45), &self.timestamp, 19)
            .add_str(Some(46), pset_change, 19)
            .build();

        if matches!(revision, 3..=7 | 998) {
            data.extend_from_slice(
                &FieldBuilder::new()
                    .add_str(Some(47), format!("PSET {}", self.pset_id), 25)
                    .add_uint(Some(48), samples.torque_unit as u64, 1)
                    .add_uint(Some(49), samples.tightening_result_type as u64, 2)
                    .build(),
            );
        }
        if matches!(revision, 4..=7 | 998) {
            data.extend_from_slice(
                &FieldBuilder::new()
                    .add_str(Some(50), &samples.identifier_part_2, 25)
                    .add_str(Some(51), &samples.identifier_part_3, 25)
                    .add_str(Some(52), &samples.identifier_part_4, 25)
                    .build(),
            );
        }
        if matches!(revision, 5..=7 | 998) {
            data.extend_from_slice(
                &FieldBuilder::new()
                    .add_str(Some(53), &samples.customer_tightening_error_code, 4)
                    .build(),
            );
        }
        if revision >= 6 {
            data.extend_from_slice(
                &FieldBuilder::new()
                    .add_uint(Some(54), samples.prevail_torque_compensate as u64, 6)
                    .add_uint(Some(55), samples.tightening_error_status_2 as u64, 10)
                    .build(),
            );
        }
        if revision == 7 {
            data.extend_from_slice(
                &FieldBuilder::new()
                    .add_uint(Some(56), samples.compensated_angle as u64, 7)
                    .add_uint(Some(57), samples.final_angle_decimal as u64, 7)
                    .build(),
            );
        } else if revision == 998 {
            data.extend_from_slice(
                &FieldBuilder::new()
                    .add_uint(Some(56), samples.multistage_count as u64, 2)
                    .add_uint(Some(57), 1, 2)
                    .add_uint(Some(58), samples.multistage_torque as u64, 6)
                    .add_uint(None, samples.multistage_angle as u64, 5)
                    .build(),
            );
        }
        data
    }

    fn serialize_light(&self) -> Vec<u8> {
        FieldBuilder::new()
            .add_str(None, self.vin_number.as_deref().unwrap_or(""), 25)
            .add_uint(None, self.job_id as u64, 2)
            .add_uint(None, self.pset_id as u64, 3)
            .add_uint(None, self.batch_size as u64, 4)
            .add_uint(None, self.batch_counter as u64, 4)
            .add_uint(None, self.batch_status_value(), 1)
            .add_uint(None, u64::from(self.tightening_status), 1)
            .add_uint(None, u64::from(self.torque_status), 1)
            .add_uint(None, u64::from(self.angle_status), 1)
            .add_uint(None, (self.torque * 100.0) as u64, 6)
            .add_uint(None, self.angle as u64, 5)
            .add_str(None, &self.timestamp, 19)
            .add_str(None, self.last_pset_change.as_deref().unwrap_or(""), 19)
            .add_uint(None, self.tightening_id.unwrap_or(0) as u64, 10)
            .build()
    }
}

impl ResponseData for TighteningResult {
    fn serialize(&self) -> Vec<u8> {
        self.serialize_revision_1()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tightening_result_serialization() {
        let result = TighteningResult::example();
        let data = ResponseData::serialize(&result);

        // Should contain multiple parameters
        assert!(!data.is_empty());
        assert!(data.len() > 100); // Complex structure should be large
    }

    #[test]
    fn revision_seven_contains_all_extension_fields() {
        let data =
            TighteningResult::example().serialize_revision(7, &ProtocolSampleData::default());
        let text = String::from_utf8(data).unwrap();
        assert!(text.contains("47PSET 1"));
        assert!(text.contains("53"));
        assert!(text.contains("54000025"));
        assert!(text.contains("560003950"));
        assert!(text.contains("570003950"));
    }

    #[test]
    fn revision_998_contains_stage_result() {
        let data =
            TighteningResult::example().serialize_revision(998, &ProtocolSampleData::default());
        let text = String::from_utf8(data).unwrap();
        assert!(text.ends_with("560157015800123000040"));
    }

    #[test]
    fn revision_999_uses_compact_layout_without_parameter_ids() {
        let data =
            TighteningResult::example().serialize_revision(999, &ProtocolSampleData::default());
        assert_eq!(data.len(), 101);
        assert!(
            String::from_utf8(data)
                .unwrap()
                .starts_with("TEST123456789")
        );
    }

    #[test]
    fn revision_payload_lengths_match_r28_tables() {
        let result = TighteningResult::example();
        let samples = ProtocolSampleData::default();
        for (revision, expected_length) in [
            (1, 211),
            (2, 365),
            (3, 399),
            (4, 480),
            (5, 486),
            (6, 506),
            (7, 524),
            (998, 527),
            (999, 101),
        ] {
            assert_eq!(
                result.serialize_revision(revision, &samples).len(),
                expected_length,
                "revision {revision}"
            );
        }
    }
}
