use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;
use crate::protocol::revision::MidRevision;
use crate::pset::Pset;

/// MID 0015 - Parameter Set Selected
///
/// Notification sent when a parameter set is selected
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PsetSelected {
    /// Parameter Set ID that was selected
    pub pset_id: u32,
    pub pset_name: String,
    pub last_change: String,
    pub rotation_direction: u8,
    pub batch_size: u32,
    pub torque_min: f64,
    pub torque_max: f64,
    pub torque_target: f64,
    pub angle_min: f64,
    pub angle_max: f64,
    pub angle_target: f64,
    pub first_target: f64,
    pub start_final_angle: f64,
}

impl PsetSelected {
    #[allow(dead_code)]
    pub fn new(pset_id: u32) -> Self {
        Self {
            pset_id,
            pset_name: format!("PSET {pset_id}"),
            last_change: chrono::Local::now().format("%Y-%m-%d:%H:%M:%S").to_string(),
            rotation_direction: 1,
            batch_size: 0,
            torque_min: 0.0,
            torque_max: 0.0,
            torque_target: 0.0,
            angle_min: 0.0,
            angle_max: 0.0,
            angle_target: 0.0,
            first_target: 0.0,
            start_final_angle: 0.0,
        }
    }

    pub fn from_pset(pset: &Pset, batch_size: u32) -> Self {
        Self {
            pset_id: pset.id,
            pset_name: pset.name.clone(),
            last_change: chrono::Local::now().format("%Y-%m-%d:%H:%M:%S").to_string(),
            rotation_direction: 1,
            batch_size,
            torque_min: pset.torque_min,
            torque_max: pset.torque_max,
            torque_target: (pset.torque_min + pset.torque_max) / 2.0,
            angle_min: pset.angle_min,
            angle_max: pset.angle_max,
            angle_target: (pset.angle_min + pset.angle_max) / 2.0,
            first_target: pset.torque_min,
            start_final_angle: pset.torque_min,
        }
    }

    pub fn serialize_revision(&self, revision: MidRevision) -> Vec<u8> {
        if revision == 1 {
            return FieldBuilder::new()
                .add_int(None, self.pset_id as i32, 3)
                .add_str(None, &self.last_change, 19)
                .build();
        }

        FieldBuilder::new()
            .add_int(Some(1), self.pset_id as i32, 3)
            .add_str(Some(2), &self.pset_name, 25)
            .add_str(Some(3), &self.last_change, 19)
            .add_int(Some(4), self.rotation_direction as i32, 1)
            .add_int(Some(5), self.batch_size as i32, 2)
            .add_int(Some(6), (self.torque_min * 100.0) as i32, 6)
            .add_int(Some(7), (self.torque_max * 100.0) as i32, 6)
            .add_int(Some(8), (self.torque_target * 100.0) as i32, 6)
            .add_int(Some(9), self.angle_min as i32, 5)
            .add_int(Some(10), self.angle_max as i32, 5)
            .add_int(Some(11), self.angle_target as i32, 5)
            .add_int(Some(12), (self.first_target * 100.0) as i32, 6)
            .add_int(Some(13), (self.start_final_angle * 100.0) as i32, 6)
            .build()
    }
}

impl ResponseData for PsetSelected {
    fn serialize(&self) -> Vec<u8> {
        self.serialize_revision(1)
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
        assert_eq!(data.len(), 22);
        assert_eq!(&data[..3], b"005");
    }

    #[test]
    fn test_pset_selected_large_id() {
        let pset = PsetSelected::new(123);
        let data = pset.serialize();
        assert_eq!(&data[..3], b"123");
    }

    #[test]
    fn revision_two_contains_extended_pset_data() {
        let pset = Pset {
            id: 2,
            name: "Wheel Bolt".to_string(),
            torque_min: 10.0,
            torque_max: 14.0,
            angle_min: 30.0,
            angle_max: 50.0,
            description: None,
        };
        let selected = PsetSelected::from_pset(&pset, 4);
        let data = String::from_utf8(selected.serialize_revision(2)).unwrap();
        assert!(data.starts_with("01002"));
        assert!(data.contains("02Wheel Bolt"));
        assert!(data.contains("0504"));
        assert!(data.contains("06001000"));
        assert!(data.contains("08001200"));
    }
}
