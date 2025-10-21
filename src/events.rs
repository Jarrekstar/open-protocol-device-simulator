use crate::handler::data::TighteningResult;
use crate::multi_spindle::{MultiSpindleResult, MultiSpindleStatus};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Events that can be broadcast to all connected clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum SimulatorEvent {
    /// A tightening operation was completed
    TighteningCompleted { result: TighteningResult },

    /// A parameter set was selected
    PsetChanged { pset_id: u32, pset_name: String },

    /// Tool state changed (enabled/disabled)
    ToolStateChanged { enabled: bool },

    /// Batch was completed
    BatchCompleted { total: u32 },

    /// Vehicle ID was changed
    VehicleIdChanged { vin: String },

    /// Multi-spindle status update completed
    MultiSpindleStatusCompleted { status: MultiSpindleStatus },

    /// Multi-spindle tightening result completed
    MultiSpindleResultCompleted { result: MultiSpindleResult },

    /// Auto-tightening progress update
    AutoTighteningProgress {
        counter: u32,
        target_size: u32,
        running: bool,
    },
}

/// Type alias for the event broadcaster (sender side)
pub type EventBroadcaster = broadcast::Sender<SimulatorEvent>;

/// Type alias for event receivers (subscriber side)
#[allow(dead_code)]
pub type EventReceiver = broadcast::Receiver<SimulatorEvent>;
