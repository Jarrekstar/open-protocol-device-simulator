use crate::batch_manager::TighteningInfo;
use crate::config::DeviceConfig;
use crate::device_fsm::DeviceFSMState;
use crate::failure_simulator::FailureConfig;
use crate::job::{Job, JobProgress, JobRuntimeState, JobStatus};
use crate::multi_spindle::MultiSpindleConfig;
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
    pub current_job_name: Option<String>,
    pub current_job_status: Option<JobStatus>,
    pub current_job_step: Option<u32>,
    pub current_job_step_progress: u32,
    pub current_job_step_batch_size: u32,
    pub current_job_total_progress: u32,
    pub current_job_total_steps: u32,
    pub current_job_total_batch_size: u32,

    // Multi-spindle configuration
    pub multi_spindle_config: MultiSpindleConfig,

    // Communication failure injection configuration
    pub failure_config: FailureConfig,
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
            current_job_name: None,
            current_job_status: None,
            current_job_step: None,
            current_job_step_progress: 0,
            current_job_step_batch_size: 0,
            current_job_total_progress: 0,
            current_job_total_steps: 0,
            current_job_total_batch_size: 0,
            multi_spindle_config: MultiSpindleConfig::default(),
            failure_config: FailureConfig::default(),
        }
    }

    /// Create a new device state from configuration
    pub fn new_from_config(config: &DeviceConfig) -> Self {
        Self {
            cell_id: config.cell_id,
            channel_id: config.channel_id,
            controller_name: config.controller_name.clone(),
            supplier_code: config.supplier_code.clone(),
            current_pset_id: Some(1),
            current_pset_name: Some("Default".to_string()),
            tightening_tracker: TighteningTracker::new(),
            device_fsm_state: DeviceFSMState::idle(),
            tool_enabled: true,
            vehicle_id: None,
            current_job_id: Some(1),
            current_job_name: None,
            current_job_status: None,
            current_job_step: None,
            current_job_step_progress: 0,
            current_job_step_batch_size: 0,
            current_job_total_progress: 0,
            current_job_total_steps: 0,
            current_job_total_batch_size: 0,
            multi_spindle_config: MultiSpindleConfig::default(),
            failure_config: FailureConfig::default(),
        }
    }

    /// Create a thread-safe shared state
    pub fn new_shared() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new()))
    }

    /// Create a thread-safe shared state from configuration
    pub fn new_shared_from_config(config: &DeviceConfig) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new_from_config(config)))
    }

    /// Set the parameter set
    pub fn set_pset(&mut self, pset_id: u32, pset_name: Option<String>) {
        self.current_pset_id = Some(pset_id);
        self.current_pset_name = pset_name;
    }

    pub fn select_job(&mut self, job: Job, pset_name: Option<String>) -> Result<(), String> {
        if self.tightening_tracker.is_job_running() {
            return Err("A Job is already running".to_string());
        }
        let first_pset_id = job
            .steps
            .first()
            .ok_or_else(|| "A Job must contain at least one step".to_string())?
            .pset_id;
        self.tightening_tracker.start_job(job);
        self.set_pset(first_pset_id, pset_name);
        self.tool_enabled = true;
        self.refresh_job_fields();
        Ok(())
    }

    pub fn restart_job(&mut self, job_id: u32, pset_name: Option<String>) -> Result<(), String> {
        if !self.tightening_tracker.restart_job(job_id) {
            return Err("Job not running".to_string());
        }
        let first_pset_id = self
            .tightening_tracker
            .job_execution()
            .expect("Job execution exists after restart")
            .current_step()
            .pset_id;
        self.set_pset(first_pset_id, pset_name);
        self.tool_enabled = true;
        self.refresh_job_fields();
        Ok(())
    }

    pub fn clear_job_mode(&mut self) -> Result<(), String> {
        if self.tightening_tracker.is_job_running() {
            return Err("Cannot clear JobMode while a Job is running".to_string());
        }
        self.tightening_tracker.exit_job();
        self.current_job_id = None;
        self.current_job_name = None;
        self.current_job_status = None;
        self.current_job_step = None;
        self.current_job_step_progress = 0;
        self.current_job_step_batch_size = 0;
        self.current_job_total_progress = 0;
        self.current_job_total_steps = 0;
        self.current_job_total_batch_size = 0;
        Ok(())
    }

    pub fn is_job_mode(&self) -> bool {
        self.tightening_tracker.is_job_mode()
    }

    pub fn is_job_running(&self) -> bool {
        self.tightening_tracker.is_job_running()
    }

    pub fn job_runtime_state(&self) -> Option<JobRuntimeState> {
        self.tightening_tracker.job_runtime_state()
    }

    pub fn add_tightening(&mut self, ok: bool) -> TighteningInfo {
        let info = self.tightening_tracker.add_tightening(ok);
        self.apply_job_progress(info.job_progress.as_ref());
        info
    }

    fn apply_job_progress(&mut self, progress: Option<&JobProgress>) {
        if let Some(progress) = progress {
            if progress.step_changed
                && let Some(execution) = self.tightening_tracker.job_execution()
            {
                self.current_pset_id = Some(execution.current_step().pset_id);
                self.current_pset_name = None;
            }
            if progress.completed_status.is_some()
                && let Some(execution) = self.tightening_tracker.job_execution()
                && execution.job.lock_at_job_done
                && !execution.job.repeat_job
            {
                self.tool_enabled = false;
            }
            self.refresh_job_fields();
        }
    }

    pub fn refresh_job_fields(&mut self) {
        let Some(runtime) = self.tightening_tracker.job_runtime_state() else {
            return;
        };
        self.current_job_id = Some(runtime.job_id);
        self.current_job_name = Some(runtime.job_name);
        self.current_job_status = Some(runtime.status);
        self.current_job_step = Some(runtime.current_step);
        self.current_job_step_progress = runtime.step_progress;
        self.current_job_step_batch_size = runtime.step_batch_size;
        self.current_job_total_progress = runtime.total_progress;
        self.current_job_total_steps = runtime.total_steps;
        self.current_job_total_batch_size = runtime.total_batch_size;
    }

    /// Set batch size (enables batch mode)
    pub fn set_batch_size(&mut self, size: u32) {
        self.tightening_tracker.enable_batch(size);
    }

    /// Increment batch counter without tightening (MID 0128 - skip bolt)
    pub fn increment_batch(&mut self) -> u32 {
        self.increment_batch_with_progress().0
    }

    pub fn increment_batch_with_progress(&mut self) -> (u32, Option<JobProgress>) {
        if let Some(progress) = self.tightening_tracker.increment_with_job_progress() {
            let counter = progress.step_counter;
            self.apply_job_progress(Some(&progress));
            (counter, Some(progress))
        } else {
            (self.tightening_tracker.increment_batch(), None)
        }
    }

    /// Reset batch counter (MID 0020)
    /// Returns true if in batch mode, false otherwise
    pub fn reset_batch(&mut self) -> bool {
        self.tightening_tracker.reset_batch()
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

    /// Enable multi-spindle mode
    pub fn enable_multi_spindle(&mut self, spindle_count: u8, sync_id: u32) -> Result<(), String> {
        let config = MultiSpindleConfig::new(spindle_count, sync_id);
        if !config.is_valid() {
            return Err(format!(
                "Invalid multi-spindle configuration: spindle_count must be 2-16, got {}",
                spindle_count
            ));
        }
        self.multi_spindle_config = config;
        Ok(())
    }

    /// Disable multi-spindle mode (revert to single-spindle)
    pub fn disable_multi_spindle(&mut self) {
        self.multi_spindle_config = MultiSpindleConfig::disable();
    }

    /// Check if multi-spindle mode is enabled
    ///
    /// Query method for checking multi-spindle state.
    /// Used by webUI dashboard to display mode and by HTTP API endpoints
    /// for status reporting and configuration validation.
    #[allow(dead_code)]
    pub fn is_multi_spindle_enabled(&self) -> bool {
        self.multi_spindle_config.enabled
    }

    /// Get multi-spindle configuration
    ///
    /// Query method for accessing multi-spindle settings.
    /// Used by webUI configuration panel to display and edit spindle
    /// count and sync ID settings, and by HTTP API for configuration export.
    #[allow(dead_code)]
    pub fn get_multi_spindle_config(&self) -> &MultiSpindleConfig {
        &self.multi_spindle_config
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
        assert!(state.tool_enabled);
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
        assert!(!state.tool_enabled);
        state.enable_tool();
        assert!(state.tool_enabled);
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

    fn test_job(lock_at_done: bool) -> Job {
        Job {
            id: 7,
            name: "Runtime".to_string(),
            forced_order: 1,
            first_tightening_timeout: 0,
            job_timeout: 0,
            batch_count_mode: 0,
            lock_at_job_done: lock_at_done,
            use_line_control: false,
            repeat_job: false,
            loosening_mode: 0,
            repair_mode: 0,
            steps: vec![crate::job::JobStep {
                channel_id: 1,
                pset_id: 1,
                auto_value: true,
                batch_size: 1,
            }],
        }
    }

    #[test]
    fn job_completion_locks_tool_and_restart_resets_progress() {
        let mut state = DeviceState::new();
        state
            .select_job(test_job(true), Some("Light Duty".to_string()))
            .unwrap();
        state.add_tightening(true);
        assert_eq!(state.current_job_status, Some(JobStatus::Ok));
        assert!(!state.tool_enabled);

        state
            .restart_job(7, Some("Light Duty".to_string()))
            .unwrap();
        assert_eq!(state.current_job_status, Some(JobStatus::Running));
        assert_eq!(state.current_job_total_progress, 0);
        assert!(state.tool_enabled);
    }

    #[test]
    fn job_mode_mid_0128_style_increment_advances_progress() {
        let mut state = DeviceState::new();
        state
            .select_job(test_job(false), Some("Light Duty".to_string()))
            .unwrap();
        let (counter, progress) = state.increment_batch_with_progress();
        assert_eq!(counter, 1);
        assert!(progress.is_some());
        assert_eq!(state.current_job_status, Some(JobStatus::Ok));
    }
}
