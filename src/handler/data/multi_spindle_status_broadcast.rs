use crate::multi_spindle::MultiSpindleStatus;
use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;

/// MID 0091 - Multi-spindle status broadcast
/// Sent to subscribed clients when multi-spindle status changes
pub struct MultiSpindleStatusBroadcast {
    pub status: MultiSpindleStatus,
}

impl MultiSpindleStatusBroadcast {
    pub fn new(status: MultiSpindleStatus) -> Self {
        Self { status }
    }

    /// Create broadcast from raw sync_id and status parameters
    ///
    /// Convenience constructor for creating status broadcasts from raw values.
    /// Used by webUI multi-spindle simulation controls to manually trigger
    /// status broadcasts with specific parameters for testing.
    #[allow(dead_code)]
    pub fn from_sync_id(sync_id: u32, spindle_count: u8, status_code: u8) -> Self {
        let status = MultiSpindleStatus {
            sync_id,
            status: status_code,
            spindle_count,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        Self { status }
    }
}

impl ResponseData for MultiSpindleStatusBroadcast {
    fn serialize(&self) -> Vec<u8> {
        // MID 0091 Revision 1 format:
        // - Sync tightening ID (4 digits)
        // - Status (1 digit): 0=Waiting, 1=Running, 2=Completed
        // - Spindle count (2 digits)
        // - Timestamp (19 chars): YYYY-MM-DD HH:MM:SS

        FieldBuilder::new()
            .add_int(None, self.status.sync_id as i32, 4)
            .add_int(None, self.status.status as i32, 1)
            .add_int(None, self.status.spindle_count as i32, 2)
            .add_str(None, &self.status.timestamp, 19)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_spindle_status_broadcast_serialization() {
        let status = MultiSpindleStatus {
            sync_id: 123,
            status: 1, // Running
            spindle_count: 4,
            timestamp: "2024-01-15 14:30:45".to_string(),
        };

        let broadcast = MultiSpindleStatusBroadcast::new(status);
        let data = broadcast.serialize();

        // Verify structure: sync_id(4) + status(1) + count(2) + timestamp(19)
        let data_str = String::from_utf8_lossy(&data);

        // sync_id should be "0123"
        assert_eq!(&data_str[0..4], "0123");

        // status should be "1"
        assert_eq!(&data_str[4..5], "1");

        // spindle_count should be "04"
        assert_eq!(&data_str[5..7], "04");

        // timestamp should be the full timestamp
        assert_eq!(&data_str[7..26], "2024-01-15 14:30:45");
    }

    #[test]
    fn test_multi_spindle_status_broadcast_waiting() {
        let broadcast = MultiSpindleStatusBroadcast::from_sync_id(100, 2, 0);
        let data = broadcast.serialize();
        let data_str = String::from_utf8_lossy(&data);

        assert_eq!(&data_str[0..4], "0100"); // sync_id
        assert_eq!(&data_str[4..5], "0"); // Waiting status
        assert_eq!(&data_str[5..7], "02"); // 2 spindles
    }

    #[test]
    fn test_multi_spindle_status_broadcast_completed() {
        let broadcast = MultiSpindleStatusBroadcast::from_sync_id(999, 16, 2);
        let data = broadcast.serialize();
        let data_str = String::from_utf8_lossy(&data);

        assert_eq!(&data_str[0..4], "0999"); // sync_id
        assert_eq!(&data_str[4..5], "2"); // Completed status
        assert_eq!(&data_str[5..7], "16"); // 16 spindles
    }

    #[test]
    fn test_multi_spindle_status_broadcast_length() {
        let broadcast = MultiSpindleStatusBroadcast::from_sync_id(1, 2, 1);
        let data = broadcast.serialize();

        // Total: 4 + 1 + 2 + 19 = 26 bytes
        assert_eq!(data.len(), 26);
    }
}
