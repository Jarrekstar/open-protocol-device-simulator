// Library exports for integration testing
pub mod batch_manager;
pub mod codec;
pub mod device_fsm;
pub mod events;
pub mod failure_simulator;
pub mod handler;
pub mod http_server;
pub mod multi_spindle;
pub mod observable_state;
pub mod protocol;
pub mod pset;
pub mod session;
pub mod state;
pub mod subscriptions;
pub mod tightening_tracker;

// Re-export commonly used types
pub use events::SimulatorEvent;
pub use observable_state::ObservableState;
pub use state::DeviceState;
