use crate::job::{Job, JobRuntimeState};
use crate::protocol::field::FieldBuilder;
use crate::protocol::revision::ProtocolSampleData;
use std::sync::Arc;

pub trait JobRevisionCodec: Send + Sync {
    fn revision(&self) -> u16;
    fn parse_job_id(&self, data: &[u8]) -> Result<u32, String>;
    fn serialize_job_ids(&self, jobs: &[Job]) -> Result<Vec<u8>, String>;
    fn serialize_job_data(&self, job: &Job) -> Result<Vec<u8>, String>;
    fn serialize_job_info(
        &self,
        state: &JobRuntimeState,
        samples: &ProtocolSampleData,
    ) -> Result<Vec<u8>, String>;
}

#[derive(Debug)]
pub struct JobCodec {
    revision: u16,
}

impl JobCodec {
    fn new(revision: u16) -> Self {
        Self { revision }
    }

    fn id_width(&self) -> usize {
        if self.revision == 1 { 2 } else { 4 }
    }
}

impl JobRevisionCodec for JobCodec {
    fn revision(&self) -> u16 {
        self.revision
    }

    fn parse_job_id(&self, data: &[u8]) -> Result<u32, String> {
        let width = self.id_width();
        if data.len() != width || !data.iter().all(u8::is_ascii_digit) {
            return Err(format!(
                "Revision {} Job ID must contain exactly {width} ASCII digits",
                self.revision
            ));
        }
        std::str::from_utf8(data)
            .map_err(|_| "Job ID is not valid ASCII".to_string())?
            .parse::<u32>()
            .map_err(|_| "Job ID is invalid".to_string())
    }

    fn serialize_job_ids(&self, jobs: &[Job]) -> Result<Vec<u8>, String> {
        let width = self.id_width();
        let maximum = if width == 2 { 99 } else { 9_999 };
        if jobs.len() > maximum || jobs.iter().any(|job| job.id > maximum as u32) {
            return Err(format!(
                "Revision {} supports at most {maximum} Jobs with {width}-digit IDs",
                self.revision
            ));
        }
        let mut jobs = jobs.to_vec();
        jobs.sort_by_key(|job| job.id);
        let mut data = format!("{:0width$}", jobs.len(), width = width).into_bytes();
        for job in jobs {
            data.extend_from_slice(format!("{:0width$}", job.id, width = width).as_bytes());
        }
        Ok(data)
    }

    fn serialize_job_data(&self, job: &Job) -> Result<Vec<u8>, String> {
        if self.revision > 3 {
            return Err(format!(
                "MID 0033 does not define revision {}",
                self.revision
            ));
        }
        let id_width = self.id_width();
        let maximum_id = if id_width == 2 { 99 } else { 9_999 };
        if job.id > maximum_id || job.steps.len() > 50 {
            return Err(format!(
                "Job cannot be represented by revision {}",
                self.revision
            ));
        }

        let mut data = FieldBuilder::new()
            .add_uint(Some(1), job.id as u64, id_width)
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
        for (index, step) in job.steps.iter().enumerate() {
            if step.channel_id > 99 || step.pset_id > 999 || step.batch_size > 99 {
                return Err(format!(
                    "Job step cannot be represented by revision {}",
                    self.revision
                ));
            }
            let serialized = if self.revision < 3 {
                format!(
                    "{:02}:{:03}:{}:{:02};",
                    step.channel_id,
                    step.pset_id,
                    u8::from(step.auto_value),
                    step.batch_size
                )
            } else {
                let step_name = format!("Step {}", index + 1);
                format!(
                    "{:02}:{:03}:{}:{:02}:{:02}:{:<25}:{:02};",
                    step.channel_id,
                    step.pset_id,
                    u8::from(step.auto_value),
                    step.batch_size,
                    step.channel_id,
                    step_name,
                    1
                )
            };
            data.extend_from_slice(serialized.as_bytes());
        }
        Ok(data)
    }

    fn serialize_job_info(
        &self,
        state: &JobRuntimeState,
        samples: &ProtocolSampleData,
    ) -> Result<Vec<u8>, String> {
        let id_width = self.id_width();
        let maximum_id = if id_width == 2 { 99 } else { 9_999 };
        if state.job_id > maximum_id
            || state.total_batch_size > 9_999
            || state.total_progress > 9_999
        {
            return Err(format!(
                "Job state cannot be represented by revision {}",
                self.revision
            ));
        }

        let mut data = FieldBuilder::new()
            .add_uint(Some(1), state.job_id as u64, id_width)
            .add_int(Some(2), state.status.protocol_value() as i32, 1)
            .add_int(Some(3), state.batch_count_mode as i32, 1)
            .add_uint(Some(4), state.total_batch_size as u64, 4)
            .add_uint(Some(5), state.total_progress as u64, 4)
            .add_str(
                Some(6),
                state.timestamp.format("%Y-%m-%d:%H:%M:%S").to_string(),
                19,
            )
            .build();

        if self.revision == 3 {
            data.extend_from_slice(
                &FieldBuilder::new()
                    .add_uint(Some(7), state.current_step as u64, 3)
                    .add_uint(Some(8), state.total_steps as u64, 3)
                    .add_uint(Some(9), 1, 2)
                    .build(),
            );
        } else if self.revision >= 4 {
            data.extend_from_slice(
                &FieldBuilder::new()
                    .add_uint(Some(7), 0, 3)
                    .add_uint(Some(8), 0, 3)
                    .add_uint(Some(9), 0, 2)
                    .build(),
            );
        }
        if self.revision >= 4 {
            data.extend_from_slice(
                &FieldBuilder::new()
                    .add_uint(Some(10), samples.job_tightening_status as u64, 2)
                    .build(),
            );
        }
        if self.revision >= 5 {
            data.extend_from_slice(
                &FieldBuilder::new()
                    .add_uint(Some(11), samples.job_sequence_number as u64, 5)
                    .add_str(Some(12), &samples.identifier_part_1, 25)
                    .add_str(Some(13), &samples.identifier_part_2, 25)
                    .add_str(Some(14), &samples.identifier_part_3, 25)
                    .add_str(Some(15), &samples.identifier_part_4, 25)
                    .build(),
            );
        }
        Ok(data)
    }
}

pub fn codec_for_revision(revision: u16) -> Option<Arc<dyn JobRevisionCodec>> {
    match revision {
        1..=5 => Some(Arc::new(JobCodec::new(revision))),
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
            JobCodec::new(1)
                .serialize_job_ids(&[job(), second])
                .unwrap(),
            b"020102"
        );
    }

    #[test]
    fn serializes_mid_0033_revision_1_layout() {
        let data = JobCodec::new(1).serialize_job_data(&job()).unwrap();
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
            JobCodec::new(1)
                .serialize_job_info(&state, &ProtocolSampleData::default())
                .unwrap(),
            b"0101020030040008050003062001-12-01:20:12:45"
        );
    }

    #[test]
    fn rejects_invalid_job_id_width_and_digits() {
        assert!(JobCodec::new(1).parse_job_id(b"1").is_err());
        assert!(JobCodec::new(1).parse_job_id(b"A1").is_err());
    }

    #[test]
    fn revision_two_uses_four_digit_job_ids() {
        let mut extended = job();
        extended.id = 1_234;
        assert_eq!(
            JobCodec::new(2).serialize_job_ids(&[extended]).unwrap(),
            b"00011234"
        );
        assert_eq!(JobCodec::new(2).parse_job_id(b"1234").unwrap(), 1_234);
    }

    #[test]
    fn revision_three_serializes_extended_step_fields() {
        let data = JobCodec::new(3).serialize_job_data(&job()).unwrap();
        let text = String::from_utf8(data).unwrap();
        assert!(text.ends_with("15:011:0:22:15:Step 1                   :01;"));
    }

    #[test]
    fn revision_five_adds_sequence_and_identifiers() {
        let state = JobRuntimeState {
            job_id: 1,
            job_name: "Wheel".to_string(),
            status: JobStatus::Running,
            batch_count_mode: 0,
            current_step: 1,
            total_steps: 2,
            current_pset_id: 11,
            step_progress: 3,
            step_batch_size: 8,
            total_progress: 3,
            total_batch_size: 8,
            timestamp: Utc.with_ymd_and_hms(2001, 12, 1, 20, 12, 45).unwrap(),
        };
        let samples = ProtocolSampleData::default();
        let text = String::from_utf8(
            JobCodec::new(5)
                .serialize_job_info(&state, &samples)
                .unwrap(),
        )
        .unwrap();
        assert!(text.contains("1100001"));
        assert!(text.contains("13WORKORDER-0001"));
    }
}
