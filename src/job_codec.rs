use crate::job::{Job, JobRuntimeState};
use crate::protocol::field::FieldBuilder;
use std::sync::Arc;

pub trait JobRevisionCodec: Send + Sync {
    fn revision(&self) -> u8;
    fn parse_job_id(&self, data: &[u8]) -> Result<u32, String>;
    fn serialize_job_ids(&self, jobs: &[Job]) -> Result<Vec<u8>, String>;
    fn serialize_job_data(&self, job: &Job) -> Result<Vec<u8>, String>;
    fn serialize_job_info(&self, state: &JobRuntimeState) -> Result<Vec<u8>, String>;
}

#[derive(Default)]
pub struct JobRevision1Codec;

impl JobRevisionCodec for JobRevision1Codec {
    fn revision(&self) -> u8 {
        1
    }

    fn parse_job_id(&self, data: &[u8]) -> Result<u32, String> {
        if data.len() != 2 || !data.iter().all(u8::is_ascii_digit) {
            return Err("Revision 1 Job ID must contain exactly two ASCII digits".to_string());
        }
        std::str::from_utf8(data)
            .map_err(|_| "Job ID is not valid ASCII".to_string())?
            .parse::<u32>()
            .map_err(|_| "Job ID is invalid".to_string())
    }

    fn serialize_job_ids(&self, jobs: &[Job]) -> Result<Vec<u8>, String> {
        if jobs.len() > 99 || jobs.iter().any(|job| job.id > 99) {
            return Err("Revision 1 supports at most 99 Jobs with IDs 00-99".to_string());
        }
        let mut jobs = jobs.to_vec();
        jobs.sort_by_key(|job| job.id);
        let mut data = format!("{:02}", jobs.len()).into_bytes();
        for job in jobs {
            data.extend_from_slice(format!("{:02}", job.id).as_bytes());
        }
        Ok(data)
    }

    fn serialize_job_data(&self, job: &Job) -> Result<Vec<u8>, String> {
        if job.id > 99 || job.steps.len() > 50 {
            return Err("Job cannot be represented by revision 1".to_string());
        }

        let mut data = FieldBuilder::new()
            .add_int(Some(1), job.id as i32, 2)
            .add_str(Some(2), &job.name, 25)
            .add_int(Some(3), job.forced_order as i32, 1)
            .add_int(Some(4), job.first_tightening_timeout as i32, 4)
            .add_int(Some(5), job.job_timeout as i32, 5)
            .add_int(Some(6), job.batch_count_mode as i32, 1)
            .add_int(Some(7), job.lock_at_job_done as i32, 1)
            .add_int(Some(8), job.use_line_control as i32, 1)
            .add_int(Some(9), job.repeat_job as i32, 1)
            .add_int(Some(10), job.loosening_mode as i32, 1)
            .add_int(Some(11), job.repair_mode as i32, 1)
            .add_int(Some(12), job.steps.len() as i32, 2)
            .build();
        data.extend_from_slice(b"13");
        for step in &job.steps {
            if step.channel_id > 99 || step.pset_id > 999 || step.batch_size > 99 {
                return Err("Job step cannot be represented by revision 1".to_string());
            }
            data.extend_from_slice(
                format!(
                    "{:02}:{:03}:{}:{:02};",
                    step.channel_id,
                    step.pset_id,
                    u8::from(step.auto_value),
                    step.batch_size
                )
                .as_bytes(),
            );
        }
        Ok(data)
    }

    fn serialize_job_info(&self, state: &JobRuntimeState) -> Result<Vec<u8>, String> {
        if state.job_id > 99 || state.total_batch_size > 9_999 || state.total_progress > 9_999 {
            return Err("Job state cannot be represented by revision 1".to_string());
        }
        Ok(FieldBuilder::new()
            .add_int(Some(1), state.job_id as i32, 2)
            .add_int(Some(2), state.status.protocol_value() as i32, 1)
            .add_int(Some(3), state.batch_count_mode as i32, 1)
            .add_int(Some(4), state.total_batch_size as i32, 4)
            .add_int(Some(5), state.total_progress as i32, 4)
            .add_str(
                Some(6),
                state.timestamp.format("%Y-%m-%d:%H:%M:%S").to_string(),
                19,
            )
            .build())
    }
}

pub fn codec_for_revision(revision: u8) -> Option<Arc<dyn JobRevisionCodec>> {
    match revision {
        1 => Some(Arc::new(JobRevision1Codec)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::{JobStatus, JobStep};
    use chrono::{TimeZone, Utc};

    fn job() -> Job {
        Job {
            id: 1,
            name: "Wheel".to_string(),
            forced_order: 1,
            first_tightening_timeout: 12,
            job_timeout: 345,
            batch_count_mode: 0,
            lock_at_job_done: true,
            use_line_control: false,
            repeat_job: false,
            loosening_mode: 2,
            repair_mode: 0,
            steps: vec![JobStep {
                channel_id: 15,
                pset_id: 11,
                auto_value: false,
                batch_size: 22,
            }],
        }
    }

    #[test]
    fn serializes_mid_0031_revision_1_example() {
        let mut second = job();
        second.id = 2;
        assert_eq!(
            JobRevision1Codec
                .serialize_job_ids(&[job(), second])
                .unwrap(),
            b"020102"
        );
    }

    #[test]
    fn serializes_mid_0033_revision_1_layout() {
        let data = JobRevision1Codec.serialize_job_data(&job()).unwrap();
        assert_eq!(
            data,
            b"010102Wheel                    031040012050034506007108009010211012011315:011:0:22;"
        );
    }

    #[test]
    fn serializes_mid_0035_revision_1_layout() {
        let state = JobRuntimeState {
            job_id: 1,
            job_name: "Wheel".to_string(),
            status: JobStatus::Running,
            batch_count_mode: 0,
            current_step: 1,
            total_steps: 1,
            current_pset_id: 11,
            step_progress: 3,
            step_batch_size: 8,
            total_progress: 3,
            total_batch_size: 8,
            timestamp: Utc.with_ymd_and_hms(2001, 12, 1, 20, 12, 45).unwrap(),
        };
        assert_eq!(
            JobRevision1Codec.serialize_job_info(&state).unwrap(),
            b"0101022030040008050003062001-12-01:20:12:45"
        );
    }

    #[test]
    fn rejects_invalid_job_id_width_and_digits() {
        assert!(JobRevision1Codec.parse_job_id(b"1").is_err());
        assert!(JobRevision1Codec.parse_job_id(b"A1").is_err());
    }
}
