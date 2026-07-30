pub mod abort_job;
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
pub mod pset_selected_ack;
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
use crate::protocol::revision::{MidRevision, ProtocolConfiguration};
use crate::protocol::{Message, Response};
use crate::state::DeviceState;
use crate::subscriptions::Subscriptions;
use crate::tightening_tracker::OperationMode;
use std::collections::{HashMap, HashSet};
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
    handlers: HashMap<(u16, MidRevision), Box<dyn MidHandler>>,
    known_mids: HashSet<u16>,
    protocol_configuration: ProtocolConfiguration,
    operation_mode_state: Option<Arc<RwLock<DeviceState>>>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self::with_protocol_configuration(ProtocolConfiguration::default())
    }

    pub fn with_protocol_configuration(protocol_configuration: ProtocolConfiguration) -> Self {
        Self {
            handlers: HashMap::new(),
            known_mids: HashSet::new(),
            protocol_configuration,
            operation_mode_state: None,
        }
    }

    /// Register a handler for a specific MID
    pub fn register(&mut self, mid: u16, handler: Box<dyn MidHandler>) {
        self.register_revision(mid, 1, handler);
    }

    pub fn register_revision(
        &mut self,
        mid: u16,
        revision: MidRevision,
        handler: Box<dyn MidHandler>,
    ) {
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
        if !self
            .protocol_configuration
            .accepts(message.mid, message.revision)
        {
            return Ok(HandlerResult::Response(Response::from_data(
                4,
                1,
                data::ErrorResponse::new(
                    message.mid,
                    data::error_response::ErrorCode::MidRevisionUnsupported,
                ),
            )));
        }
        if let Some(state) = &self.operation_mode_state {
            let current_mode = state.read().unwrap().operation_mode();
            if let Some(error_code) = mode_rejection(message.mid, current_mode) {
                println!("MID {:04} rejected in {:?} mode", message.mid, current_mode);
                return Ok(HandlerResult::Response(Response::from_data(
                    4,
                    message.revision,
                    data::ErrorResponse::new(message.mid, error_code),
                )));
            }
        }
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

/// Profile-conditional command gating per Open Protocol R2.8.0 semantics.
///
/// Upload requests and subscriptions are never gated: integrators request
/// IDs and subscribe at startup (spec §3.7.2/§3.7.3) before any production
/// style is chosen, so MID 0030-0037 must answer in every profile. Batch
/// configuration (MID 0019/0020) is a runtime attribute of a parameter set,
/// not a mode, so it is accepted in the Pset profile too.
fn mode_rejection(mid: u16, mode: OperationMode) -> Option<data::error_response::ErrorCode> {
    use data::error_response::ErrorCode;
    match mid {
        // Direct Pset selection is locked while the controller is governed
        // by Jobs: error 03 "Parameter set can not be set".
        18 if mode == OperationMode::Job => Some(ErrorCode::PsetCannotBeSet),
        // The Job definition owns batch sizes while the Job profile is
        // active; the shared tracker cannot be reconfigured mid-Job.
        19 | 20 if mode == OperationMode::Job => Some(ErrorCode::InvalidData),
        // Outside the Job profile the controller behaves as "Job off"
        // (MID 0130): Job selection answers error 20 "Job can not be set".
        // MID 0039 restart is not gated; without a running Job its handler
        // answers error 21.
        38 if mode != OperationMode::Job => Some(ErrorCode::JobCannotBeSet),
        // MID 0128 advances either a running Job or a configured PSET batch.
        // Without either profile it answers error 21 "Job not running".
        128 if !matches!(mode, OperationMode::Job | OperationMode::Batch) => {
            Some(ErrorCode::JobNotRunning)
        }
        // Abort Job (MID 0127) is deliberately never gated: §3.7.2 lists it
        // under Pset-style production control too, so it must be accepted in
        // every operation mode and always answers MID 0005 (no error reply).
        _ => None,
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
    create_registry_with_repositories_and_protocol(
        observable_state,
        pset_repository,
        job_repository,
        ProtocolConfiguration::default(),
    )
}

pub fn create_registry_with_repositories_and_protocol(
    observable_state: ObservableState,
    pset_repository: crate::pset::SharedPsetRepository,
    job_repository: crate::job::SharedJobRepository,
    protocol_configuration: ProtocolConfiguration,
) -> HandlerRegistry {
    let mut registry = HandlerRegistry::with_protocol_configuration(protocol_configuration.clone());
    let state = observable_state.state();
    registry.operation_mode_state = Some(Arc::clone(state));

    // Register all MID handlers (sorted by MID number)
    for revision in 1..=6 {
        registry.register_revision(
            1,
            revision,
            Box::new(communication_start::CommunicationStartHandler::new(
                Arc::clone(state),
                protocol_configuration.clone(),
            )),
        );
    }
    registry.register(
        3,
        Box::new(communication_stop::CommunicationStopHandler::new()),
    );
    for revision in 1..=2 {
        registry.register_revision(
            14,
            revision,
            Box::new(pset_subscription::PsetSubscriptionHandler),
        );
        registry.register_revision(
            16,
            revision,
            Box::new(pset_selected_ack::PsetSelectedAckHandler),
        );
        registry.register_revision(
            17,
            revision,
            Box::new(pset_unsubscribe::PsetUnsubscribeHandler),
        );
    }
    registry.register(
        18,
        Box::new(pset_select::PsetSelectHandler::new(
            observable_state.clone(),
            pset_repository.clone(),
        )),
    );
    registry.register(
        19,
        Box::new(batch_size::BatchSizeHandler::new(
            observable_state.clone(),
            Arc::clone(&pset_repository),
        )),
    );
    registry.register(
        20,
        Box::new(batch_reset::BatchResetHandler::new(Arc::clone(state))),
    );
    registry.register(
        127,
        Box::new(abort_job::AbortJobHandler::new(observable_state.clone())),
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
    for revision in 1..=2 {
        registry.register_revision(
            50,
            revision,
            Box::new(vehicle_id_download::VehicleIdDownloadHandler::new(
                observable_state.clone(),
            )),
        );
        registry.register_revision(
            51,
            revision,
            Box::new(vehicle_id_subscription::VehicleIdSubscriptionHandler),
        );
        registry.register_revision(53, revision, Box::new(vehicle_id_ack::VehicleIdAckHandler));
        registry.register_revision(
            54,
            revision,
            Box::new(vehicle_id_unsubscribe::VehicleIdUnsubscribeHandler),
        );
    }
    registry.register(
        90,
        Box::new(multi_spindle_status_subscribe::MultiSpindleStatusSubscribeHandler),
    );
    registry.register(
        92,
        Box::new(multi_spindle_status_ack::MultiSpindleStatusAckHandler),
    );
    registry.register(
        93,
        Box::new(multi_spindle_status_unsubscribe::MultiSpindleStatusUnsubscribeHandler),
    );
    for revision in 1..=5 {
        registry.register_revision(
            100,
            revision,
            Box::new(multi_spindle_result_subscribe::MultiSpindleResultSubscribeHandler),
        );
        registry.register_revision(
            102,
            revision,
            Box::new(multi_spindle_result_ack::MultiSpindleResultAckHandler),
        );
        registry.register_revision(
            103,
            revision,
            Box::new(multi_spindle_result_unsubscribe::MultiSpindleResultUnsubscribeHandler),
        );
    }
    for revision in [1, 2, 3, 4, 5, 6, 7, 998, 999] {
        registry.register_revision(
            60,
            revision,
            Box::new(tightening_result_subscription::TighteningResultSubscriptionHandler),
        );
        registry.register_revision(
            62,
            revision,
            Box::new(tightening_result_ack::TighteningResultAckHandler),
        );
        registry.register_revision(
            63,
            revision,
            Box::new(tightening_result_unsubscribe::TighteningResultUnsubscribeHandler),
        );
    }
    registry.register(9999, Box::new(keep_alive::KeepAliveHandler));

    for revision in 1..=2 {
        let codec = crate::job_codec::codec_for_revision(revision).expect("Job ID codec");
        registry.register_revision(
            30,
            revision,
            Box::new(job::JobIdUploadHandler::new(job_repository.clone(), codec)),
        );
    }
    registry.mark_known(31);
    for revision in 1..=3 {
        let codec = crate::job_codec::codec_for_revision(revision).expect("Job data codec");
        registry.register_revision(
            32,
            revision,
            Box::new(job::JobDataUploadHandler::new(
                job_repository.clone(),
                codec,
            )),
        );
    }
    registry.mark_known(33);
    registry.mark_known(35);
    for revision in 1..=5 {
        registry.register_revision(34, revision, Box::new(job::JobInfoSubscribeHandler));
        registry.register_revision(36, revision, Box::new(job::JobInfoAcknowledgeHandler));
        registry.register_revision(37, revision, Box::new(job::JobInfoUnsubscribeHandler));
    }
    for revision in 1..=2 {
        let codec = crate::job_codec::codec_for_revision(revision).expect("Job command codec");
        registry.register_revision(
            38,
            revision,
            Box::new(job::JobSelectHandler::new(
                observable_state.clone(),
                job_repository.clone(),
                pset_repository.clone(),
                codec.clone(),
            )),
        );
        registry.register_revision(
            39,
            revision,
            Box::new(job::JobRestartHandler::new(
                observable_state.clone(),
                pset_repository.clone(),
                codec,
            )),
        );
    }

    registry
}
