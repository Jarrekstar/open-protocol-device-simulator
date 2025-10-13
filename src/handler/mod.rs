pub mod batch_size;
pub mod communication_start;
pub mod data;
pub mod keep_alive;
pub mod pset_select;
pub mod pset_subscription;
pub mod pset_unsubscribe;
pub mod tightening_result_subscription;
pub mod tightening_result_unsubscribe;
pub mod tightening_result_ack;
pub mod tool_disable;
pub mod tool_enable;
pub mod vehicle_id;

use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Unknown MID: {0}")]
    UnknownMid(u16),

    #[error("Handler error: {0}")]
    #[allow(dead_code)]
    Processing(String),
}

/// Trait for handling specific MID messages
pub trait MidHandler: Send + Sync {
    /// Process a message and generate a response
    fn handle(&self, message: &Message) -> Result<Response, HandlerError>;
}

/// Registry that routes MIDs to their handlers
pub struct HandlerRegistry {
    handlers: HashMap<u16, Box<dyn MidHandler>>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for a specific MID
    pub fn register(&mut self, mid: u16, handler: Box<dyn MidHandler>) {
        self.handlers.insert(mid, handler);
    }

    /// Process a message using the appropriate handler
    pub fn handle_message(&self, message: &Message) -> Result<Response, HandlerError> {
        let handler = self
            .handlers
            .get(&message.mid)
            .ok_or(HandlerError::UnknownMid(message.mid))?;

        handler.handle(message)
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a registry with all standard handlers registered
pub fn create_default_registry(state: Arc<RwLock<DeviceState>>) -> HandlerRegistry {
    let mut registry = HandlerRegistry::new();

    // Register all MID handlers (sorted by MID number)
    registry.register(1, Box::new(communication_start::CommunicationStartHandler::new(Arc::clone(&state))));
    registry.register(14, Box::new(pset_subscription::PsetSubscriptionHandler::new(Arc::clone(&state))));
    registry.register(16, Box::new(pset_unsubscribe::PsetUnsubscribeHandler::new(Arc::clone(&state))));
    registry.register(18, Box::new(pset_select::PsetSelectHandler::new(Arc::clone(&state))));
    registry.register(19, Box::new(batch_size::BatchSizeHandler::new(Arc::clone(&state))));
    registry.register(42, Box::new(tool_disable::ToolDisableHandler::new(Arc::clone(&state))));
    registry.register(43, Box::new(tool_enable::ToolEnableHandler::new(Arc::clone(&state))));
    registry.register(52, Box::new(vehicle_id::VehicleIdHandler::new(Arc::clone(&state))));
    registry.register(60, Box::new(tightening_result_subscription::TighteningResultSubscriptionHandler::new(Arc::clone(&state))));
    registry.register(62, Box::new(tightening_result_ack::TighteningResultAckHandler));
    registry.register(63, Box::new(tightening_result_unsubscribe::TighteningResultUnsubscribeHandler::new(Arc::clone(&state))));
    registry.register(9999, Box::new(keep_alive::KeepAliveHandler));

    registry
}
