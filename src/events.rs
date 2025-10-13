use crate::handler::data::TighteningResult;
use tokio::sync::broadcast;

/// Events that can be broadcast to all connected clients
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SimulatorEvent {
    /// A tightening operation was completed
    TighteningCompleted { result: TighteningResult },

    /// A parameter set was selected
    PsetChanged { pset_id: u32 },

    /// Tool state changed (enabled/disabled)
    ToolStateChanged { enabled: bool },

    /// Batch was completed
    BatchCompleted { total: u32 },
}

/// Type alias for the event broadcaster (sender side)
pub type EventBroadcaster = broadcast::Sender<SimulatorEvent>;

/// Type alias for event receivers (subscriber side)
#[allow(dead_code)]
pub type EventReceiver = broadcast::Receiver<SimulatorEvent>;
