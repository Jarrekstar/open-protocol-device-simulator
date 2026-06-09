pub mod batch_increment;
pub mod batch_reset;
pub mod batch_size;
pub mod communication_start;
pub mod communication_stop;
pub mod data;
pub mod job;
pub mod keep_alive;
pub mod multi_spindle_result_ack;
pub mod multi_spindle_result_subscribe;
pub mod multi_spindle_result_unsubscribe;
pub mod multi_spindle_status_ack;
pub mod multi_spindle_status_subscribe;
pub mod multi_spindle_status_unsubscribe;
pub mod pset_select;
pub mod pset_subscription;
pub mod pset_unsubscribe;
pub mod tightening_result_ack;
pub mod tightening_result_subscription;
pub mod tightening_result_unsubscribe;
pub mod tool_disable;
pub mod tool_enable;
pub mod vehicle_id_ack;
pub mod vehicle_id_download;
pub mod vehicle_id_subscription;
pub mod vehicle_id_unsubscribe;

use crate::observable_state::ObservableState;
use crate::protocol::{Message, Response};
use crate::subscriptions::Subscriptions;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Unknown MID: {0}")]
    UnknownMid(u16),

    #[error("Handler error: {0}")]
    #[allow(dead_code)]
    Processing(String),
}

#[derive(Debug)]
pub enum HandlerResult {
    Response(Response),
    NoResponse,
}

pub struct HandlerContext<'a> {
    pub subscriptions: &'a mut Subscriptions,
}

impl<'a> HandlerContext<'a> {
    pub fn new(subscriptions: &'a mut Subscriptions) -> Self {
        Self { subscriptions }
    }
}

/// Trait for handling specific MID messages
pub trait MidHandler: Send + Sync {
    /// Process a message and generate a response
    fn handle(&self, message: &Message) -> Result<Response, HandlerError>;

    fn handle_with_context(
        &self,
        message: &Message,
        _context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        self.handle(message).map(HandlerResult::Response)
    }
}

/// Registry that routes MIDs to their handlers
pub struct HandlerRegistry {
    handlers: HashMap<(u16, u8), Box<dyn MidHandler>>,
    known_mids: HashSet<u16>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            known_mids: HashSet::new(),
        }
    }

    /// Register a handler for a specific MID
    pub fn register(&mut self, mid: u16, handler: Box<dyn MidHandler>) {
        self.register_revision(mid, 1, handler);
    }

    pub fn register_revision(&mut self, mid: u16, revision: u8, handler: Box<dyn MidHandler>) {
        self.known_mids.insert(mid);
        self.handlers.insert((mid, revision), handler);
    }

    pub fn mark_known(&mut self, mid: u16) {
        self.known_mids.insert(mid);
    }

    /// Process a message using the appropriate handler
    pub fn handle_message(&self, message: &Message) -> Result<Response, HandlerError> {
        let mut subscriptions = Subscriptions::new();
        match self.dispatch(message, &mut subscriptions)? {
            HandlerResult::Response(response) => Ok(response),
            HandlerResult::NoResponse => Err(HandlerError::Processing(format!(
                "MID {:04} does not produce a response",
                message.mid
            ))),
        }
    }

    pub fn dispatch(
        &self,
        message: &Message,
        subscriptions: &mut Subscriptions,
    ) -> Result<HandlerResult, HandlerError> {
        if let Some(handler) = self.handlers.get(&(message.mid, message.revision)) {
            return handler.handle_with_context(message, &mut HandlerContext::new(subscriptions));
        }

        let code = if self.known_mids.contains(&message.mid) {
            data::error_response::ErrorCode::MidRevisionUnsupported
        } else {
            data::error_response::ErrorCode::GenericError
        };
        Ok(HandlerResult::Response(Response::from_data(
            4,
            message.revision,
            data::ErrorResponse::new(message.mid, code),
        )))
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a registry with all standard handlers registered
pub fn create_default_registry(observable_state: ObservableState) -> HandlerRegistry {
    create_registry_with_repositories(
        observable_state,
        crate::pset::create_default_repository(),
        crate::job::create_default_repository(),
    )
}

pub fn create_registry_with_repositories(
    observable_state: ObservableState,
    pset_repository: crate::pset::SharedPsetRepository,
    job_repository: crate::job::SharedJobRepository,
) -> HandlerRegistry {
    let mut registry = HandlerRegistry::new();
    let state = observable_state.state();

    // Register all MID handlers (sorted by MID number)
    registry.register(
        1,
        Box::new(communication_start::CommunicationStartHandler::new(
            Arc::clone(state),
        )),
    );
    registry.register(
        3,
        Box::new(communication_stop::CommunicationStopHandler::new()),
    );
    registry.register(14, Box::new(pset_subscription::PsetSubscriptionHandler));
    registry.register(17, Box::new(pset_unsubscribe::PsetUnsubscribeHandler));
    registry.register(
        18,
        Box::new(pset_select::PsetSelectHandler::new(
            observable_state.clone(),
            pset_repository.clone(),
        )),
    );
    registry.register(
        19,
        Box::new(batch_size::BatchSizeHandler::new(Arc::clone(state))),
    );
    registry.register(
        20,
        Box::new(batch_reset::BatchResetHandler::new(Arc::clone(state))),
    );
    registry.register(
        128,
        Box::new(batch_increment::BatchIncrementHandler::new(
            observable_state.clone(),
            pset_repository.clone(),
        )),
    );
    registry.register(
        42,
        Box::new(tool_disable::ToolDisableHandler::new(
            observable_state.clone(),
        )),
    );
    registry.register(
        43,
        Box::new(tool_enable::ToolEnableHandler::new(
            observable_state.clone(),
        )),
    );
    registry.register(
        50,
        Box::new(vehicle_id_download::VehicleIdDownloadHandler::new(
            observable_state.clone(),
        )),
    );
    registry.register(
        51,
        Box::new(vehicle_id_subscription::VehicleIdSubscriptionHandler),
    );
    registry.register(53, Box::new(vehicle_id_ack::VehicleIdAckHandler));
    registry.register(
        54,
        Box::new(vehicle_id_unsubscribe::VehicleIdUnsubscribeHandler),
    );
    registry.register(
        90,
        Box::new(multi_spindle_status_subscribe::MultiSpindleStatusSubscribeHandler),
    );
    registry.register(
        92,
        Box::new(multi_spindle_status_unsubscribe::MultiSpindleStatusUnsubscribeHandler),
    );
    registry.register(
        93,
        Box::new(multi_spindle_status_ack::MultiSpindleStatusAckHandler),
    );
    registry.register(
        100,
        Box::new(multi_spindle_result_subscribe::MultiSpindleResultSubscribeHandler),
    );
    registry.register(
        102,
        Box::new(multi_spindle_result_ack::MultiSpindleResultAckHandler),
    );
    registry.register(
        103,
        Box::new(multi_spindle_result_unsubscribe::MultiSpindleResultUnsubscribeHandler),
    );
    registry.register(
        60,
        Box::new(tightening_result_subscription::TighteningResultSubscriptionHandler),
    );
    registry.register(
        62,
        Box::new(tightening_result_ack::TighteningResultAckHandler),
    );
    registry.register(
        63,
        Box::new(tightening_result_unsubscribe::TighteningResultUnsubscribeHandler),
    );
    registry.register(9999, Box::new(keep_alive::KeepAliveHandler));

    let revision_1 = crate::job_codec::codec_for_revision(1).expect("Revision 1 Job codec");
    registry.register_revision(
        30,
        1,
        Box::new(job::JobIdUploadHandler::new(
            job_repository.clone(),
            revision_1.clone(),
        )),
    );
    registry.mark_known(31);
    registry.register_revision(
        32,
        1,
        Box::new(job::JobDataUploadHandler::new(
            job_repository.clone(),
            revision_1.clone(),
        )),
    );
    registry.mark_known(33);
    registry.register_revision(34, 1, Box::new(job::JobInfoSubscribeHandler));
    registry.mark_known(35);
    registry.register_revision(36, 1, Box::new(job::JobInfoAcknowledgeHandler));
    registry.register_revision(37, 1, Box::new(job::JobInfoUnsubscribeHandler));
    registry.register_revision(
        38,
        1,
        Box::new(job::JobSelectHandler::new(
            observable_state.clone(),
            job_repository,
            pset_repository.clone(),
            revision_1.clone(),
        )),
    );
    registry.register_revision(
        39,
        1,
        Box::new(job::JobRestartHandler::new(
            observable_state,
            pset_repository,
            revision_1,
        )),
    );

    registry
}
