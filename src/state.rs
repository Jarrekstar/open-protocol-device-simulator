use crate::device_fsm::DeviceFSMState;
use crate::tightening_tracker::TighteningTracker;
use serde::Serialize;
use std::sync::{Arc, RwLock};

/// Represents the internal state of the simulated device
#[derive(Debug, Clone, Serialize)]
pub struct DeviceState {
    // Controller identification
    pub cell_id: u32,
    pub channel_id: u32,
    pub controller_name: String,
    pub supplier_code: String,

    // Parameter set (pset) state
    pub current_pset_id: Option<u32>,
    pub current_pset_name: Option<String>,

    // Tightening tracking (single mode or batch mode)
    pub tightening_tracker: TighteningTracker,

    // Device operational state
    pub device_fsm_state: DeviceFSMState,

    // Tool state
    pub tool_enabled: bool,

    // Vehicle/Job identification
    pub vehicle_id: Option<String>,
    pub current_job_id: Option<u32>,
}

impl DeviceState {
    /// Create a new device state with default values
    pub fn new() -> Self {
        Self {
            cell_id: 1,
            channel_id: 1,
            controller_name: "OpenProtocolSimulator".to_string(),
            supplier_code: "SIM".to_string(),
            current_pset_id: Some(1),
            current_pset_name: Some("Default".to_string()),
            tightening_tracker: TighteningTracker::new(),
            device_fsm_state: DeviceFSMState::idle(),
            tool_enabled: true,
            vehicle_id: None,
            current_job_id: Some(1),
        }
    }

    /// Create a thread-safe shared state
    pub fn new_shared() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new()))
    }

    /// Set the current parameter set
    pub fn set_pset(&mut self, pset_id: u32, pset_name: Option<String>) {
        self.current_pset_id = Some(pset_id);
        self.current_pset_name = pset_name;
    }

    /// Set batch size (enables batch mode)
    pub fn set_batch_size(&mut self, size: u32) {
        self.tightening_tracker.enable_batch(size);
    }

    /// Enable the tool
    pub fn enable_tool(&mut self) {
        self.tool_enabled = true;
    }

    /// Disable the tool
    pub fn disable_tool(&mut self) {
        self.tool_enabled = false;
    }

    /// Set vehicle ID
    pub fn set_vehicle_id(&mut self, vin: String) {
        self.vehicle_id = Some(vin);
    }

    /// Clear vehicle ID
    #[allow(dead_code)]
    pub fn clear_vehicle_id(&mut self) {
        self.vehicle_id = None;
    }
}

impl Default for DeviceState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_state_creation() {
        let state = DeviceState::new();
        assert_eq!(state.cell_id, 1);
        assert_eq!(state.tool_enabled, true);
        assert_eq!(state.tightening_tracker.counter(), 0);
    }

    #[test]
    fn test_tightening_tracker() {
        let mut state = DeviceState::new();
        // In single mode, counter stays 0
        let info = state.tightening_tracker.add_tightening(true);
        assert_eq!(info.counter, 0);

        // Enable batch mode
        state.set_batch_size(2);
        let info = state.tightening_tracker.add_tightening(true);
        assert_eq!(info.counter, 1);
    }

    #[test]
    fn test_tool_state() {
        let mut state = DeviceState::new();
        state.disable_tool();
        assert_eq!(state.tool_enabled, false);
        state.enable_tool();
        assert_eq!(state.tool_enabled, true);
    }

    #[test]
    fn test_shared_state() {
        let state = DeviceState::new_shared();
        {
            let mut s = state.write().unwrap();
            s.set_pset(5, Some("Test".to_string()));
        }
        {
            let s = state.read().unwrap();
            assert_eq!(s.current_pset_id, Some(5));
        }
    }
}
