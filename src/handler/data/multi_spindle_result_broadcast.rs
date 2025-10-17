use crate::multi_spindle::MultiSpindleResult;
use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;

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
            torque_min: 4500,  // 45.00 Nm
            torque_max: 5500,  // 55.00 Nm
            torque_target: 5000, // 50.00 Nm
            // Default angle limits (180° target, ±10° range)
            angle_min: 170,
            angle_max: 190,
            angle_target: 180,
            last_change_timestamp: chrono::Local::now()
                .format("%Y-%m-%d:%H:%M:%S")
                .to_string(),
        }
    }
}

impl ResponseData for MultiSpindleResultBroadcast {
    fn serialize(&self) -> Vec<u8> {
        // MID 0101 Revision 1, 2, 3 format
        let mut builder = FieldBuilder::new();

        // Parameter 01: Number of spindles (2 bytes)
        builder = builder.add_int(Some(1), self.result.spindle_count as i32, 2);

        // Parameter 02: VIN Number (25 bytes)
        let vin = if self.vin_number.len() >= 25 {
            self.vin_number[..25].to_string()
        } else {
            format!("{:<25}", self.vin_number)
        };
        builder = builder.add_str(Some(2), &vin, 25);

        // Parameter 03: Job ID (2 bytes)
        builder = builder.add_int(Some(3), self.job_id as i32, 2);

        // Parameter 04: Parameter set ID (3 bytes)
        builder = builder.add_int(Some(4), self.pset_id as i32, 3);

        // Parameter 05: Batch size (4 bytes)
        builder = builder.add_int(Some(5), self.batch_size as i32, 4);

        // Parameter 06: Batch counter (4 bytes)
        builder = builder.add_int(Some(6), self.batch_counter as i32, 4);

        // Parameter 07: Batch status (1 byte)
        builder = builder.add_int(Some(7), self.batch_status as i32, 1);

        // Parameter 08: Torque Min limit (6 bytes, Nm * 100)
        builder = builder.add_int(Some(8), self.torque_min, 6);

        // Parameter 09: Torque Max limit (6 bytes, Nm * 100)
        builder = builder.add_int(Some(9), self.torque_max, 6);

        // Parameter 10: Torque final target (6 bytes, Nm * 100)
        builder = builder.add_int(Some(10), self.torque_target, 6);

        // Parameter 11: Angle Min (5 bytes, degrees)
        builder = builder.add_int(Some(11), self.angle_min, 5);

        // Parameter 12: Angle Max (5 bytes, degrees)
        builder = builder.add_int(Some(12), self.angle_max, 5);

        // Parameter 13: Final Angle Target (5 bytes, degrees)
        builder = builder.add_int(Some(13), self.angle_target, 5);

        // Parameter 14: Date/time of last change (19 bytes)
        builder = builder.add_str(Some(14), &self.last_change_timestamp, 19);

        // Parameter 15: Time stamp (19 bytes)
        builder = builder.add_str(Some(15), &self.result.timestamp, 19);

        // Parameter 16: Sync tightening ID (5 bytes)
        builder = builder.add_int(Some(16), self.result.result_id as i32, 5);

        // Parameter 17: Sync overall status (1 byte)
        builder = builder.add_int(Some(17), self.result.overall_status as i32, 1);

        // Parameter 18: Spindle status (18 bytes × number of spindles)
        // Each spindle: spindle# (2) + channel (2) + overall (1) + torque_stat (1) + torque (6) + angle_stat (1) + angle (5)
        for spindle in &self.result.spindle_results {
            // Bytes 1-2: Spindle number (01-99)
            builder = builder.add_int(None, spindle.spindle_id as i32, 2);

            // Bytes 3-4: Channel ID (same as spindle ID for now)
            builder = builder.add_int(None, spindle.channel_id as i32, 2);

            // Byte 5: Individual overall status (0=NOK, 1=OK)
            let overall_status = if spindle.is_ok() { 1 } else { 0 };
            builder = builder.add_int(None, overall_status, 1);

            // Byte 6: Individual torque status (0=Low, 1=OK, 2=High)
            builder = builder.add_int(None, spindle.torque_status as i32, 1);

            // Bytes 7-12: Torque result (Nm * 100, already stored as integer)
            builder = builder.add_int(None, spindle.torque, 6);

            // Byte 13: Individual angle status (0=NOK, 1=OK)
            builder = builder.add_int(None, spindle.angle_status as i32, 1);

            // Bytes 14-18: Angle value (degrees)
            builder = builder.add_int(None, spindle.angle, 5);
        }

        builder = builder.add_int(Some(18), 0, 0); // Parameter marker for spindle status section

        builder.build()
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
            torque: 5000, // 50.00 Nm
            angle: 1800,  // 180.0 degrees
            torque_status: 1, // OK
            angle_status: 0,  // OK
        };

        let spindle2 = SpindleResult {
            spindle_id: 2,
            channel_id: 2,
            torque: 5100, // 51.00 Nm
            angle: 1850,  // 185.0 degrees
            torque_status: 1, // OK
            angle_status: 0,  // OK
        };

        let spindles = vec![spindle1, spindle2];
        let result = MultiSpindleResult::new(1, 100, spindles);

        let broadcast = MultiSpindleResultBroadcast::new(
            result,
            "TEST_VIN_12345".to_string(),
            1,   // job_id
            10,  // pset_id
            0,   // batch_size (not used)
            0,   // batch_counter
            2,   // batch_status (not used)
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
    }

    #[test]
    fn test_multi_spindle_result_broadcast_with_nok() {
        let spindle1 = SpindleResult {
            spindle_id: 1,
            channel_id: 1,
            torque: 5000,
            angle: 1800,
            torque_status: 1,
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

        let broadcast = MultiSpindleResultBroadcast::new(
            result,
            "VIN".to_string(),
            1,
            10,
            0,
            0,
            2,
        );

        let data = broadcast.serialize();
        let data_str = String::from_utf8_lossy(&data);

        // Overall status should be "0" (NOK, since spindle 2 failed)
        assert!(data_str.contains("170"));
    }
}
