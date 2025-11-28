//! Observable wrapper around DeviceState that broadcasts events on state changes
//!
//! This module provides a wrapper pattern that separates state management from
//! event broadcasting, keeping DeviceState pure while allowing automatic event
//! notifications to WebSocket clients.

use crate::events::{EventBroadcaster, SimulatorEvent};
use crate::state::DeviceState;
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
        {
            let mut state = self.state.write().unwrap();
            state.enable_tool();
        }
        let _ = self
            .broadcaster
            .send(SimulatorEvent::ToolStateChanged { enabled: true });
    }

    /// Disable the tool and broadcast the event
    pub fn disable_tool(&self) {
        {
            let mut state = self.state.write().unwrap();
            state.disable_tool();
        }
        let _ = self
            .broadcaster
            .send(SimulatorEvent::ToolStateChanged { enabled: false });
    }

    /// Set the parameter set and broadcast the event
    pub fn set_pset(&self, pset_id: u32, pset_name: Option<String>) {
        let name_for_broadcast = pset_name.clone().unwrap_or_else(|| "Unknown".to_string());
        {
            let mut state = self.state.write().unwrap();
            state.set_pset(pset_id, pset_name);
        }
        let _ = self.broadcaster.send(SimulatorEvent::PsetChanged {
            pset_id,
            pset_name: name_for_broadcast,
        });
    }

    /// Set the vehicle ID and broadcast the event
    pub fn set_vehicle_id(&self, vin: String) {
        {
            let mut state = self.state.write().unwrap();
            state.set_vehicle_id(vin.clone());
        }
        let _ = self
            .broadcaster
            .send(SimulatorEvent::VehicleIdChanged { vin });
    }

    /// Set batch size (does not broadcast an event as this is internal config)
    pub fn set_batch_size(&self, size: u32) {
        let mut state = self.state.write().unwrap();
        state.set_batch_size(size);
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
        state.enable_multi_spindle(spindle_count, sync_id)
    }

    /// Disable multi-spindle mode (does not broadcast as it's config change)
    pub fn disable_multi_spindle(&self) {
        let mut state = self.state.write().unwrap();
        state.disable_multi_spindle();
    }

    /// Broadcast a simulator event (for complex operations that need manual broadcasting)
    pub fn broadcast(&self, event: SimulatorEvent) {
        let _ = self.broadcaster.send(event);
    }

    /// Subscribe to events (returns a receiver for the event broadcaster)
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<SimulatorEvent> {
        self.broadcaster.subscribe()
    }
}
