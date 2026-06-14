//! Observable wrapper around DeviceState that broadcasts events on state changes
//!
//! This module provides a wrapper pattern that separates state management from
//! event broadcasting, keeping DeviceState pure while allowing automatic event
//! notifications to WebSocket clients.

use crate::events::{EventBroadcaster, SimulatorEvent};
use crate::job::Job;
use crate::state::DeviceState;
use crate::tightening_tracker::OperationMode;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Wrapper around DeviceState that automatically broadcasts events when state changes
#[derive(Clone)]
pub struct ObservableState {
    state: Arc<RwLock<DeviceState>>,
    broadcaster: EventBroadcaster,
}

impl ObservableState {
    /// Create a new observable state wrapper
    pub fn new(state: Arc<RwLock<DeviceState>>, broadcaster: EventBroadcaster) -> Self {
        Self { state, broadcaster }
    }

    /// Get read-only access to the underlying state
    pub fn read(&self) -> RwLockReadGuard<'_, DeviceState> {
        self.state.read().unwrap()
    }

    /// Get mutable access to the underlying state (use sparingly, prefer observable methods)
    pub fn write(&self) -> RwLockWriteGuard<'_, DeviceState> {
        self.state.write().unwrap()
    }

    /// Get direct access to the state Arc (for passing to components that need raw access)
    pub fn state(&self) -> &Arc<RwLock<DeviceState>> {
        &self.state
    }

    /// Enable the tool and broadcast the event
    pub fn enable_tool(&self) {
        let changed = {
            let mut state = self.state.write().unwrap();
            let changed = !state.tool_enabled;
            state.enable_tool();
            changed
        };
        if !changed {
            return;
        }
        println!("[STATE] tool_enabled: false -> true");
        let _ = self
            .broadcaster
            .send(SimulatorEvent::ToolStateChanged { enabled: true });
    }

    /// Disable the tool and broadcast the event
    pub fn disable_tool(&self) {
        let changed = {
            let mut state = self.state.write().unwrap();
            let changed = state.tool_enabled;
            state.disable_tool();
            changed
        };
        if !changed {
            return;
        }
        println!("[STATE] tool_enabled: true -> false");
        let _ = self
            .broadcaster
            .send(SimulatorEvent::ToolStateChanged { enabled: false });
    }

    /// Set the parameter set and broadcast the event
    pub fn set_pset(&self, pset_id: u32, pset_name: Option<String>) {
        let name_for_broadcast = pset_name.clone().unwrap_or_else(|| "Unknown".to_string());
        let (previous, previous_mode, current_mode) = {
            let mut state = self.state.write().unwrap();
            let previous = (state.current_pset_id, state.current_pset_name.clone());
            let previous_mode = state.operation_mode();
            state.set_pset(pset_id, pset_name);
            (previous, previous_mode, state.operation_mode())
        };
        self.notify_operation_mode_change(previous_mode, current_mode);
        if previous.0 != Some(pset_id) || previous.1.as_deref() != Some(&name_for_broadcast) {
            println!(
                "[STATE] active_pset: {} -> {} ({})",
                previous
                    .0
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                pset_id,
                name_for_broadcast
            );
        }
        let _ = self.broadcaster.send(SimulatorEvent::PsetChanged {
            pset_id,
            pset_name: name_for_broadcast,
        });
    }

    pub fn select_job(&self, job: Job, pset_name: Option<String>) -> Result<(), String> {
        let pset_id = job.steps[0].pset_id;
        let job_id = job.id;
        let pset_event_name = pset_name.clone().unwrap_or_else(|| "Unknown".to_string());
        let (runtime, previous_mode) = {
            let mut state = self.state.write().unwrap();
            let previous_mode = state.operation_mode();
            state.select_job(job, pset_name)?;
            (
                state
                    .job_runtime_state()
                    .expect("Job runtime exists after selection"),
                previous_mode,
            )
        };
        self.notify_operation_mode_change(previous_mode, OperationMode::Job);
        println!("[STATE] active_job: none -> {job_id}");
        let _ = self.broadcaster.send(SimulatorEvent::PsetChanged {
            pset_id,
            pset_name: pset_event_name,
        });
        let _ = self
            .broadcaster
            .send(SimulatorEvent::JobSelected { state: runtime });
        Ok(())
    }

    pub fn restart_job(&self, job_id: u32, pset_name: Option<String>) -> Result<(), String> {
        let pset_event_name = pset_name.clone().unwrap_or_else(|| "Unknown".to_string());
        let runtime = {
            let mut state = self.state.write().unwrap();
            state.restart_job(job_id, pset_name)?;
            state
                .job_runtime_state()
                .expect("Job runtime exists after restart")
        };
        let _ = self.broadcaster.send(SimulatorEvent::PsetChanged {
            pset_id: runtime.current_pset_id,
            pset_name: pset_event_name,
        });
        let _ = self
            .broadcaster
            .send(SimulatorEvent::JobRestarted { state: runtime });
        println!("[STATE] active_job restarted: {job_id}");
        Ok(())
    }

    pub fn clear_job_mode(&self) -> Result<(), String> {
        let (result, previous_mode, current_mode) = {
            let mut state = self.state.write().unwrap();
            let previous_mode = state.operation_mode();
            let result = state.clear_job_mode();
            (result, previous_mode, state.operation_mode())
        };
        if result.is_ok() {
            self.notify_operation_mode_change(previous_mode, current_mode);
            println!("[STATE] active_job cleared");
        }
        result
    }

    /// Abort the currently running Job (MID 0127) and broadcast the teardown.
    ///
    /// Returns `true` when a Job was loaded and got aborted, `false` when no
    /// Job was active. Either way MID 0127 is acknowledged by the caller, since
    /// the Open Protocol spec defines no error reply for Abort Job.
    pub fn abort_job(&self) -> bool {
        let (runtime, previous_mode, current_mode) = {
            let mut state = self.state.write().unwrap();
            let previous_mode = state.operation_mode();
            let runtime = state.abort_job();
            (runtime, previous_mode, state.operation_mode())
        };
        let Some(runtime) = runtime else {
            return false;
        };
        println!("[STATE] active_job aborted: {}", runtime.job_id);
        let _ = self
            .broadcaster
            .send(SimulatorEvent::JobAborted { state: runtime });
        self.notify_operation_mode_change(previous_mode, current_mode);
        true
    }

    pub fn set_pset_mode(&self) {
        let (previous, current) = {
            let mut state = self.state.write().unwrap();
            let previous = state.operation_mode();
            state.set_pset_mode();
            (previous, state.operation_mode())
        };
        self.notify_operation_mode_change(previous, current);
    }

    pub fn set_batch_mode(&self) {
        let (previous, current) = {
            let mut state = self.state.write().unwrap();
            let previous = state.operation_mode();
            state.set_batch_mode();
            (previous, state.operation_mode())
        };
        self.notify_operation_mode_change(previous, current);
    }

    pub fn set_job_mode(&self) {
        let (previous, current) = {
            let mut state = self.state.write().unwrap();
            let previous = state.operation_mode();
            state.set_job_mode();
            (previous, state.operation_mode())
        };
        self.notify_operation_mode_change(previous, current);
    }

    /// Set the vehicle ID and broadcast the event
    pub fn set_vehicle_id(&self, vin: String) {
        let previous = {
            let mut state = self.state.write().unwrap();
            let previous = state.vehicle_id.clone();
            state.set_vehicle_id(vin.clone());
            previous
        };
        if previous.as_deref() != Some(&vin) {
            println!(
                "[STATE] vehicle_id: {} -> {}",
                previous.as_deref().unwrap_or("none"),
                vin
            );
        }
        let _ = self
            .broadcaster
            .send(SimulatorEvent::VehicleIdChanged { vin });
    }

    /// Set the batch size of a PSET (MID 0019) and broadcast any mode change
    pub fn set_pset_batch_size(&self, pset_id: u32, size: u32) {
        let (previous_size, previous_mode, current_mode, applied) = {
            let mut state = self.state.write().unwrap();
            let previous_size = state.tightening_tracker.batch_size();
            let previous_mode = state.operation_mode();
            let applied = state.set_pset_batch_size(pset_id, size);
            (
                previous_size,
                previous_mode,
                state.operation_mode(),
                applied,
            )
        };
        if applied {
            if previous_size != size {
                println!("[STATE] batch_size for PSET {pset_id}: {previous_size} -> {size}");
            }
        } else {
            println!("[STATE] batch_size for PSET {pset_id} stored: {size} (applies on selection)");
        }
        self.notify_operation_mode_change(previous_mode, current_mode);
    }

    /// Broadcast auto-tightening progress update
    pub fn broadcast_auto_progress(&self, counter: u32, target_size: u32, running: bool) {
        let _ = self
            .broadcaster
            .send(SimulatorEvent::AutoTighteningProgress {
                counter,
                target_size,
                running,
            });
    }

    /// Enable multi-spindle mode (does not broadcast as it's config change)
    pub fn enable_multi_spindle(&self, spindle_count: u8, sync_id: u32) -> Result<(), String> {
        let mut state = self.state.write().unwrap();
        let result = state.enable_multi_spindle(spindle_count, sync_id);
        if result.is_ok() {
            println!(
                "[STATE] multi_spindle: enabled (spindles={spindle_count}, sync_id={sync_id})"
            );
        }
        result
    }

    /// Disable multi-spindle mode (does not broadcast as it's config change)
    pub fn disable_multi_spindle(&self) {
        let mut state = self.state.write().unwrap();
        let was_enabled = state.multi_spindle_config.enabled;
        state.disable_multi_spindle();
        if was_enabled {
            println!("[STATE] multi_spindle: disabled");
        }
    }

    /// Broadcast a simulator event (for complex operations that need manual broadcasting)
    pub fn broadcast(&self, event: SimulatorEvent) {
        let _ = self.broadcaster.send(event);
    }

    /// Subscribe to events (returns a receiver for the event broadcaster)
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<SimulatorEvent> {
        self.broadcaster.subscribe()
    }

    /// Log and broadcast an operation mode transition, if one happened
    fn notify_operation_mode_change(&self, previous: OperationMode, current: OperationMode) {
        if previous != current {
            println!("[STATE] operation_mode: {previous:?} -> {current:?}");
            let _ = self
                .broadcaster
                .send(SimulatorEvent::OperationModeChanged { mode: current });
        }
    }
}
