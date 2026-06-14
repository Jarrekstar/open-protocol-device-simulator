use crate::multi_spindle::MultiSpindleResult;
use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;
use crate::protocol::revision::{MidRevision, ProtocolSampleData};

/// MID 0101 - Multi-spindle result broadcast
/// Sent to subscribed clients after each sync tightening operation
/// Implements Revision 1, 2, and 3 format
pub struct MultiSpindleResultBroadcast {
    pub result: MultiSpindleResult,
    pub vin_number: String,
    pub job_id: u32,
    pub pset_id: u32,
    pub batch_size: u32,
    pub batch_counter: u32,
    pub batch_status: u8, // 0=NOK, 1=OK, 2=not used
    pub torque_min: i32,
    pub torque_max: i32,
    pub torque_target: i32,
    pub angle_min: i32,
    pub angle_max: i32,
    pub angle_target: i32,
    pub last_change_timestamp: String,
}

impl MultiSpindleResultBroadcast {
    pub fn new(
        result: MultiSpindleResult,
        vin_number: String,
        job_id: u32,
        pset_id: u32,
        batch_size: u32,
        batch_counter: u32,
        batch_status: u8,
    ) -> Self {
        Self {
            result,
            vin_number,
            job_id,
            pset_id,
            batch_size,
            batch_counter,
            batch_status,
            // Default torque limits (50.00 Nm target, ±5.00 Nm range)
            torque_min: 4500,    // 45.00 Nm
            torque_max: 5500,    // 55.00 Nm
            torque_target: 5000, // 50.00 Nm
            // Default angle limits (180° target, ±10° range)
            angle_min: 170,
            angle_max: 190,
            angle_target: 180,
            last_change_timestamp: chrono::Local::now().format("%Y-%m-%d:%H:%M:%S").to_string(),
        }
    }

    pub fn serialize_revision(
        &self,
        revision: MidRevision,
        samples: &ProtocolSampleData,
    ) -> Vec<u8> {
        if !(1..=5).contains(&revision) {
            return Vec::new();
        }

        let mut builder = FieldBuilder::new()
            .add_uint(Some(1), self.result.spindle_count as u64, 2)
            .add_str(Some(2), &self.vin_number, 25)
            .add_uint(Some(3), self.job_id as u64, 2)
            .add_uint(Some(4), self.pset_id as u64, 3)
            .add_uint(Some(5), self.batch_size as u64, 4)
            .add_uint(Some(6), self.batch_counter as u64, 4)
            .add_uint(Some(7), self.batch_status as u64, 1)
            .add_uint(Some(8), self.torque_min as u64, 6)
            .add_uint(Some(9), self.torque_max as u64, 6)
            .add_uint(Some(10), self.torque_target as u64, 6)
            .add_uint(Some(11), self.angle_min as u64, 5)
            .add_uint(Some(12), self.angle_max as u64, 5)
            .add_uint(Some(13), self.angle_target as u64, 5)
            .add_str(Some(14), &self.last_change_timestamp, 19)
            .add_str(Some(15), &self.result.timestamp, 19)
            .add_uint(Some(16), self.result.result_id as u64, 5)
            .add_uint(Some(17), u64::from(self.result.is_ok()), 1);

        builder = builder.add_str(Some(18), "", 0);
        for spindle in &self.result.spindle_results {
            builder = builder
                .add_uint(None, spindle.spindle_id as u64, 2)
                .add_uint(None, spindle.channel_id as u64, 2)
                .add_uint(None, u64::from(spindle.is_ok()), 1)
                .add_uint(None, u64::from(spindle.torque_status == 0), 1)
                .add_uint(None, spindle.torque as u64, 6)
                .add_uint(None, u64::from(spindle.angle_status == 0), 1)
                .add_uint(None, spindle.angle as u64, 5);
        }
        if revision >= 4 {
            builder = builder.add_uint(Some(19), samples.system_subtype as u64, 3);
        }
        if revision >= 5 {
            builder = builder.add_uint(Some(20), samples.job_sequence_number as u64, 5);
        }
        builder.build()
    }
}

impl ResponseData for MultiSpindleResultBroadcast {
    fn serialize(&self) -> Vec<u8> {
        self.serialize_revision(1, &ProtocolSampleData::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multi_spindle::{MultiSpindleResult, SpindleResult};

    #[test]
    fn test_multi_spindle_result_broadcast_two_spindles() {
        let spindle1 = SpindleResult {
            spindle_id: 1,
            channel_id: 1,
            torque: 5000,     // 50.00 Nm
            angle: 1800,      // 180.0 degrees
            torque_status: 0, // OK
            angle_status: 0,  // OK
        };

        let spindle2 = SpindleResult {
            spindle_id: 2,
            channel_id: 2,
            torque: 5100,     // 51.00 Nm
            angle: 1850,      // 185.0 degrees
            torque_status: 0, // OK
            angle_status: 0,  // OK
        };

        let spindles = vec![spindle1, spindle2];
        let result = MultiSpindleResult::new(1, 100, spindles);

        let broadcast = MultiSpindleResultBroadcast::new(
            result,
            "TEST_VIN_12345".to_string(),
            1,  // job_id
            10, // pset_id
            0,  // batch_size (not used)
            0,  // batch_counter
            2,  // batch_status (not used)
        );

        let data = broadcast.serialize();
        let data_str = String::from_utf8_lossy(&data);

        // Verify parameter markers and key fields
        // Parameter 01: Number of spindles should be "01" followed by "02"
        assert!(data_str.contains("0102"));

        // Parameter 02: VIN should be padded to 25 chars
        assert!(data_str.contains("02TEST_VIN_12345"));

        // Parameter 16: Sync tightening ID should be "00001"
        assert!(data_str.contains("1600001"));

        // Parameter 17: Overall status should be "1" (OK, since both spindles OK)
        assert!(data_str.contains("171"));
        assert!(data_str.contains("18010111005000101800"));
    }

    #[test]
    fn test_multi_spindle_result_broadcast_with_nok() {
        let spindle1 = SpindleResult {
            spindle_id: 1,
            channel_id: 1,
            torque: 5000,
            angle: 1800,
            torque_status: 0,
            angle_status: 0,
        };

        let spindle2 = SpindleResult {
            spindle_id: 2,
            channel_id: 2,
            torque: 4000, // Too low
            angle: 1850,
            torque_status: 0, // NOK (low)
            angle_status: 0,
        };

        let spindles = vec![spindle1, spindle2];
        let result = MultiSpindleResult::new(1, 100, spindles);

        let broadcast = MultiSpindleResultBroadcast::new(result, "VIN".to_string(), 1, 10, 0, 0, 2);

        let data = broadcast.serialize();
        let data_str = String::from_utf8_lossy(&data);

        // Overall status should be "0" (NOK, since spindle 2 failed)
        assert!(data_str.contains("170"));
    }

    #[test]
    fn revision_five_adds_system_subtype_and_job_sequence() {
        let spindle = SpindleResult::ok(1, 5_000, 180);
        let result = MultiSpindleResult::new(1, 100, vec![spindle]);
        let broadcast = MultiSpindleResultBroadcast::new(result, "VIN".to_string(), 1, 10, 0, 0, 2);
        let data = broadcast.serialize_revision(5, &ProtocolSampleData::default());
        let text = String::from_utf8(data).unwrap();
        assert!(text.ends_with("190012000001"));
    }

    #[test]
    fn revision_payload_lengths_match_r28_tables() {
        let spindles = vec![
            SpindleResult::ok(1, 5_000, 180),
            SpindleResult::ok(2, 5_100, 185),
        ];
        let result = MultiSpindleResult::new(1, 100, spindles);
        let broadcast = MultiSpindleResultBroadcast::new(result, "VIN".to_string(), 1, 10, 0, 0, 2);
        let samples = ProtocolSampleData::default();
        assert_eq!(broadcast.serialize_revision(1, &samples).len(), 190);
        assert_eq!(broadcast.serialize_revision(3, &samples).len(), 190);
        assert_eq!(broadcast.serialize_revision(4, &samples).len(), 195);
        assert_eq!(broadcast.serialize_revision(5, &samples).len(), 202);
    }
}
