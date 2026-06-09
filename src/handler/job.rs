use crate::handler::data::command_accepted::CommandAccepted;
use crate::handler::data::error_response::{ErrorCode, ErrorResponse};
use crate::handler::{HandlerContext, HandlerError, HandlerResult, MidHandler};
use crate::job::SharedJobRepository;
use crate::job_codec::JobRevisionCodec;
use crate::observable_state::ObservableState;
use crate::protocol::{Message, Response};
use crate::pset::SharedPsetRepository;
use std::sync::Arc;

fn error(message: &Message, code: ErrorCode) -> HandlerResult {
    HandlerResult::Response(Response::from_data(
        4,
        message.revision,
        ErrorResponse::new(message.mid, code),
    ))
}

pub struct JobIdUploadHandler {
    jobs: SharedJobRepository,
    codec: Arc<dyn JobRevisionCodec>,
}

impl JobIdUploadHandler {
    pub fn new(jobs: SharedJobRepository, codec: Arc<dyn JobRevisionCodec>) -> Self {
        Self { jobs, codec }
    }
}

impl MidHandler for JobIdUploadHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        let mut subscriptions = crate::subscriptions::Subscriptions::new();
        match self.handle_with_context(message, &mut HandlerContext::new(&mut subscriptions))? {
            HandlerResult::Response(response) => Ok(response),
            HandlerResult::NoResponse => Err(HandlerError::Processing("No response".to_string())),
        }
    }

    fn handle_with_context(
        &self,
        message: &Message,
        _context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        if !message.data.is_empty() {
            return Ok(error(message, ErrorCode::InvalidData));
        }
        let jobs = self.jobs.read().unwrap().get_all();
        let data = self
            .codec
            .serialize_job_ids(&jobs)
            .map_err(HandlerError::Processing)?;
        Ok(HandlerResult::Response(Response::new(
            31,
            self.codec.revision(),
            data,
        )))
    }
}

pub struct JobDataUploadHandler {
    jobs: SharedJobRepository,
    codec: Arc<dyn JobRevisionCodec>,
}

impl JobDataUploadHandler {
    pub fn new(jobs: SharedJobRepository, codec: Arc<dyn JobRevisionCodec>) -> Self {
        Self { jobs, codec }
    }
}

impl MidHandler for JobDataUploadHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        let id = self
            .codec
            .parse_job_id(&message.data)
            .map_err(HandlerError::Processing)?;
        let job = self
            .jobs
            .read()
            .unwrap()
            .get_by_id(id)
            .ok_or_else(|| HandlerError::Processing("Job not found".to_string()))?;
        let data = self
            .codec
            .serialize_job_data(&job)
            .map_err(HandlerError::Processing)?;
        Ok(Response::new(33, self.codec.revision(), data))
    }

    fn handle_with_context(
        &self,
        message: &Message,
        _context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        let id = match self.codec.parse_job_id(&message.data) {
            Ok(id) => id,
            Err(_) => return Ok(error(message, ErrorCode::InvalidData)),
        };
        let Some(job) = self.jobs.read().unwrap().get_by_id(id) else {
            return Ok(error(message, ErrorCode::JobNotFound));
        };
        let data = self
            .codec
            .serialize_job_data(&job)
            .map_err(HandlerError::Processing)?;
        Ok(HandlerResult::Response(Response::new(
            33,
            self.codec.revision(),
            data,
        )))
    }
}

pub struct JobInfoSubscribeHandler;

impl MidHandler for JobInfoSubscribeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        Ok(Response::from_data(
            5,
            message.revision,
            CommandAccepted::with_mid(34),
        ))
    }

    fn handle_with_context(
        &self,
        message: &Message,
        context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        if !message.data.is_empty() {
            return Ok(error(message, ErrorCode::InvalidData));
        }
        if !context.subscriptions.subscribe_job_info(message.revision) {
            return Ok(error(message, ErrorCode::SubscriptionAlreadyExists));
        }
        Ok(HandlerResult::Response(Response::from_data(
            5,
            message.revision,
            CommandAccepted::with_mid(34),
        )))
    }
}

pub struct JobInfoAcknowledgeHandler;

impl MidHandler for JobInfoAcknowledgeHandler {
    fn handle(&self, _message: &Message) -> Result<Response, HandlerError> {
        Err(HandlerError::Processing(
            "MID 0036 does not produce a response".to_string(),
        ))
    }

    fn handle_with_context(
        &self,
        message: &Message,
        _context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        if !message.data.is_empty() {
            return Ok(error(message, ErrorCode::InvalidData));
        }
        Ok(HandlerResult::NoResponse)
    }
}

pub struct JobInfoUnsubscribeHandler;

impl MidHandler for JobInfoUnsubscribeHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        Ok(Response::from_data(
            5,
            message.revision,
            CommandAccepted::with_mid(37),
        ))
    }

    fn handle_with_context(
        &self,
        message: &Message,
        context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        if !message.data.is_empty() {
            return Ok(error(message, ErrorCode::InvalidData));
        }
        if !context.subscriptions.unsubscribe_job_info() {
            return Ok(error(message, ErrorCode::SubscriptionDoesNotExist));
        }
        Ok(HandlerResult::Response(Response::from_data(
            5,
            message.revision,
            CommandAccepted::with_mid(37),
        )))
    }
}

pub struct JobSelectHandler {
    state: ObservableState,
    jobs: SharedJobRepository,
    psets: SharedPsetRepository,
    codec: Arc<dyn JobRevisionCodec>,
}

impl JobSelectHandler {
    pub fn new(
        state: ObservableState,
        jobs: SharedJobRepository,
        psets: SharedPsetRepository,
        codec: Arc<dyn JobRevisionCodec>,
    ) -> Self {
        Self {
            state,
            jobs,
            psets,
            codec,
        }
    }
}

impl MidHandler for JobSelectHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        let mut subscriptions = crate::subscriptions::Subscriptions::new();
        match self.handle_with_context(message, &mut HandlerContext::new(&mut subscriptions))? {
            HandlerResult::Response(response) => Ok(response),
            HandlerResult::NoResponse => Err(HandlerError::Processing("No response".to_string())),
        }
    }

    fn handle_with_context(
        &self,
        message: &Message,
        _context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        let id = match self.codec.parse_job_id(&message.data) {
            Ok(id) => id,
            Err(_) => return Ok(error(message, ErrorCode::InvalidData)),
        };
        let Some(job) = self.jobs.read().unwrap().get_by_id(id) else {
            return Ok(error(message, ErrorCode::JobNotFound));
        };
        if self.state.read().is_job_running() {
            return Ok(error(message, ErrorCode::JobCannotBeSet));
        }
        let pset_name = self
            .psets
            .read()
            .unwrap()
            .get_by_id(job.steps[0].pset_id)
            .map(|pset| pset.name);
        if self.state.select_job(job, pset_name).is_err() {
            return Ok(error(message, ErrorCode::JobCannotBeSet));
        }
        Ok(HandlerResult::Response(Response::from_data(
            5,
            message.revision,
            CommandAccepted::with_mid(38),
        )))
    }
}

pub struct JobRestartHandler {
    state: ObservableState,
    psets: SharedPsetRepository,
    codec: Arc<dyn JobRevisionCodec>,
}

impl JobRestartHandler {
    pub fn new(
        state: ObservableState,
        psets: SharedPsetRepository,
        codec: Arc<dyn JobRevisionCodec>,
    ) -> Self {
        Self {
            state,
            psets,
            codec,
        }
    }
}

impl MidHandler for JobRestartHandler {
    fn handle(&self, message: &Message) -> Result<Response, HandlerError> {
        let mut subscriptions = crate::subscriptions::Subscriptions::new();
        match self.handle_with_context(message, &mut HandlerContext::new(&mut subscriptions))? {
            HandlerResult::Response(response) => Ok(response),
            HandlerResult::NoResponse => Err(HandlerError::Processing("No response".to_string())),
        }
    }

    fn handle_with_context(
        &self,
        message: &Message,
        _context: &mut HandlerContext<'_>,
    ) -> Result<HandlerResult, HandlerError> {
        let id = match self.codec.parse_job_id(&message.data) {
            Ok(id) => id,
            Err(_) => return Ok(error(message, ErrorCode::InvalidData)),
        };
        let first_pset_id = {
            let state = self.state.read();
            let Some(execution) = state.tightening_tracker.job_execution() else {
                return Ok(error(message, ErrorCode::JobNotRunning));
            };
            if execution.job.id != id {
                return Ok(error(message, ErrorCode::JobNotRunning));
            }
            execution.job.steps[0].pset_id
        };
        let pset_name = self
            .psets
            .read()
            .unwrap()
            .get_by_id(first_pset_id)
            .map(|pset| pset.name);
        if self.state.restart_job(id, pset_name).is_err() {
            return Ok(error(message, ErrorCode::JobNotRunning));
        }
        Ok(HandlerResult::Response(Response::from_data(
            5,
            message.revision,
            CommandAccepted::with_mid(39),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::SimulatorEvent;
    use crate::handler::{HandlerResult, create_registry_with_repositories};
    use crate::job::{Job, JobStep};
    use crate::protocol::Message;
    use crate::state::DeviceState;
    use crate::subscriptions::Subscriptions;

    fn example_job() -> Job {
        Job {
            id: 7,
            name: "Job Seven".to_string(),
            forced_order: 1,
            first_tightening_timeout: 0,
            job_timeout: 0,
            batch_count_mode: 0,
            lock_at_job_done: false,
            use_line_control: false,
            repeat_job: false,
            loosening_mode: 0,
            repair_mode: 0,
            steps: vec![JobStep {
                channel_id: 1,
                pset_id: 1,
                auto_value: true,
                batch_size: 2,
            }],
        }
    }

    fn registry() -> crate::handler::HandlerRegistry {
        let state = DeviceState::new_shared();
        let (broadcaster, _) = tokio::sync::broadcast::channel::<SimulatorEvent>(16);
        let observable = ObservableState::new(state, broadcaster);
        let psets = crate::pset::create_default_repository();
        let jobs = crate::job::create_default_repository();
        jobs.write().unwrap().create(example_job()).unwrap();
        create_registry_with_repositories(observable, psets, jobs)
    }

    fn message(mid: u16, revision: u8, data: &[u8]) -> Message {
        Message {
            length: (20 + data.len()) as u32,
            mid,
            revision,
            data: data.to_vec(),
        }
    }

    #[test]
    fn lists_and_uploads_jobs() {
        let registry = registry();
        let mut subscriptions = Subscriptions::new();
        let HandlerResult::Response(list) = registry
            .dispatch(&message(30, 1, b""), &mut subscriptions)
            .unwrap()
        else {
            panic!("MID 0030 must respond");
        };
        assert_eq!(list.mid, 31);
        assert_eq!(list.data, b"0107");

        let HandlerResult::Response(data) = registry
            .dispatch(&message(32, 1, b"07"), &mut subscriptions)
            .unwrap()
        else {
            panic!("MID 0032 must respond");
        };
        assert_eq!(data.mid, 33);
        assert!(data.data.starts_with(b"0107"));
    }

    #[test]
    fn subscription_validation_and_acknowledgement() {
        let registry = registry();
        let mut subscriptions = Subscriptions::new();
        let HandlerResult::Response(first) = registry
            .dispatch(&message(34, 1, b""), &mut subscriptions)
            .unwrap()
        else {
            panic!("MID 0034 must respond");
        };
        assert_eq!(first.mid, 5);
        assert_eq!(subscriptions.job_info_revision(), Some(1));

        let HandlerResult::Response(duplicate) = registry
            .dispatch(&message(34, 1, b""), &mut subscriptions)
            .unwrap()
        else {
            panic!("Duplicate MID 0034 must respond");
        };
        assert_eq!(duplicate.mid, 4);
        assert_eq!(duplicate.data, b"003418");

        assert!(matches!(
            registry
                .dispatch(&message(36, 1, b""), &mut subscriptions)
                .unwrap(),
            HandlerResult::NoResponse
        ));
    }

    #[test]
    fn returns_job_errors_and_revision_errors() {
        let registry = registry();
        let mut subscriptions = Subscriptions::new();
        let HandlerResult::Response(missing) = registry
            .dispatch(&message(38, 1, b"99"), &mut subscriptions)
            .unwrap()
        else {
            panic!("Missing Job must respond");
        };
        assert_eq!(missing.data, b"003817");

        let HandlerResult::Response(revision) = registry
            .dispatch(&message(30, 2, b""), &mut subscriptions)
            .unwrap()
        else {
            panic!("Unsupported revision must respond");
        };
        assert_eq!(revision.data, b"003097");
    }

    #[test]
    fn unsubscribe_requires_existing_subscription() {
        let registry = registry();
        let mut subscriptions = Subscriptions::new();
        let HandlerResult::Response(response) = registry
            .dispatch(&message(37, 1, b""), &mut subscriptions)
            .unwrap()
        else {
            panic!("MID 0037 must respond");
        };
        assert_eq!(response.data, b"003719");
    }

    #[test]
    fn job_subscriptions_are_isolated_per_connection() {
        let registry = registry();
        let mut first = Subscriptions::new();
        let second = Subscriptions::new();
        registry.dispatch(&message(34, 1, b""), &mut first).unwrap();
        assert_eq!(first.job_info_revision(), Some(1));
        assert_eq!(second.job_info_revision(), None);
    }
}
