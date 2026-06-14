use chrono::{DateTime, Utc};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobStep {
    pub channel_id: u32,
    pub pset_id: u32,
    pub auto_value: bool,
    pub batch_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub id: u32,
    pub name: String,
    pub forced_order: u8,
    pub first_tightening_timeout: u32,
    pub job_timeout: u32,
    pub batch_count_mode: u8,
    pub lock_at_job_done: bool,
    pub use_line_control: bool,
    pub repeat_job: bool,
    pub loosening_mode: u8,
    pub repair_mode: u8,
    pub steps: Vec<JobStep>,
}

impl Job {
    pub fn validate(&self, configured_channels: &[u32], pset_ids: &[u32]) -> Result<(), String> {
        if self.id > 9_999 {
            return Err("Job ID must be in the range 0000-9999".to_string());
        }
        if self.name.len() > 25 || !self.name.bytes().all(|byte| (b' '..=b'~').contains(&byte)) {
            return Err(
                "Job name must use printable ASCII and be no longer than 25 bytes".to_string(),
            );
        }
        if self.name.trim().is_empty() {
            return Err("Job name is required".to_string());
        }
        if self.forced_order > 2 {
            return Err("forced_order must be 0, 1, or 2".to_string());
        }
        if self.first_tightening_timeout > 9_999 {
            return Err("first_tightening_timeout must be in the range 0000-9999".to_string());
        }
        if self.job_timeout > 99_999 {
            return Err("job_timeout must be in the range 00000-99999".to_string());
        }
        if self.batch_count_mode > 1 {
            return Err("batch_count_mode must be 0 or 1".to_string());
        }
        if self.loosening_mode > 2 {
            return Err("loosening_mode must be 0, 1, or 2".to_string());
        }
        if self.repair_mode > 1 {
            return Err("repair_mode must be 0 or 1".to_string());
        }
        if !(1..=50).contains(&self.steps.len()) {
            return Err("A Job must contain between 1 and 50 steps".to_string());
        }

        let channels: HashSet<u32> = configured_channels.iter().copied().collect();
        let psets: HashSet<u32> = pset_ids.iter().copied().collect();
        for (index, step) in self.steps.iter().enumerate() {
            if step.channel_id > 99 || !channels.contains(&step.channel_id) {
                return Err(format!(
                    "Step {} references unconfigured channel {}",
                    index + 1,
                    step.channel_id
                ));
            }
            if step.pset_id > 999 || !psets.contains(&step.pset_id) {
                return Err(format!(
                    "Step {} references missing PSET {}",
                    index + 1,
                    step.pset_id
                ));
            }
            if !(1..=99).contains(&step.batch_size) {
                return Err(format!(
                    "Step {} batch_size must be in the range 01-99",
                    index + 1
                ));
            }
        }

        Ok(())
    }

    pub fn total_batch_size(&self) -> u32 {
        self.steps.iter().map(|step| step.batch_size).sum()
    }
}

pub trait JobRepository: Send + Sync {
    fn get_all(&self) -> Vec<Job>;
    fn get_by_id(&self, id: u32) -> Option<Job>;
    fn create(&mut self, job: Job) -> Result<Job, String>;
    fn update(&mut self, id: u32, job: Job) -> Result<Job, String>;
    fn delete(&mut self, id: u32) -> Result<(), String>;
    fn references_pset(&self, pset_id: u32) -> bool;
}

#[derive(Default)]
pub struct InMemoryJobRepository {
    jobs: Vec<Job>,
}

impl InMemoryJobRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl JobRepository for InMemoryJobRepository {
    fn get_all(&self) -> Vec<Job> {
        let mut jobs = self.jobs.clone();
        jobs.sort_by_key(|job| job.id);
        jobs
    }

    fn get_by_id(&self, id: u32) -> Option<Job> {
        self.jobs.iter().find(|job| job.id == id).cloned()
    }

    fn create(&mut self, job: Job) -> Result<Job, String> {
        if self.jobs.iter().any(|existing| existing.id == job.id) {
            return Err(format!("Job with id {:02} already exists", job.id));
        }
        self.jobs.push(job.clone());
        Ok(job)
    }

    fn update(&mut self, id: u32, mut job: Job) -> Result<Job, String> {
        let existing = self
            .jobs
            .iter_mut()
            .find(|existing| existing.id == id)
            .ok_or_else(|| format!("Job with id {:02} not found", id))?;
        job.id = id;
        *existing = job.clone();
        Ok(job)
    }

    fn delete(&mut self, id: u32) -> Result<(), String> {
        let original_len = self.jobs.len();
        self.jobs.retain(|job| job.id != id);
        if self.jobs.len() == original_len {
            Err(format!("Job with id {:02} not found", id))
        } else {
            Ok(())
        }
    }

    fn references_pset(&self, pset_id: u32) -> bool {
        self.jobs
            .iter()
            .any(|job| job.steps.iter().any(|step| step.pset_id == pset_id))
    }
}

pub struct SqliteJobRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteJobRepository {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let manager = SqliteConnectionManager::file(db_path)
            .with_init(|connection| connection.execute_batch("PRAGMA foreign_keys = ON;"));
        let pool = Pool::new(manager).map_err(|error| format!("Failed to create pool: {error}"))?;
        let repository = Self { pool };
        repository.init_schema()?;
        Ok(repository)
    }

    fn init_schema(&self) -> Result<(), String> {
        let connection = self
            .pool
            .get()
            .map_err(|error| format!("Failed to get connection: {error}"))?;
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS jobs (
                    id INTEGER PRIMARY KEY CHECK (id BETWEEN 0 AND 9999),
                    name TEXT NOT NULL,
                    forced_order INTEGER NOT NULL,
                    first_tightening_timeout INTEGER NOT NULL,
                    job_timeout INTEGER NOT NULL,
                    batch_count_mode INTEGER NOT NULL,
                    lock_at_job_done INTEGER NOT NULL,
                    use_line_control INTEGER NOT NULL,
                    repeat_job INTEGER NOT NULL,
                    loosening_mode INTEGER NOT NULL,
                    repair_mode INTEGER NOT NULL,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                );
                CREATE TABLE IF NOT EXISTS job_steps (
                    job_id INTEGER NOT NULL,
                    step_order INTEGER NOT NULL,
                    channel_id INTEGER NOT NULL,
                    pset_id INTEGER NOT NULL,
                    auto_value INTEGER NOT NULL,
                    batch_size INTEGER NOT NULL,
                    PRIMARY KEY (job_id, step_order),
                    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE,
                    FOREIGN KEY (pset_id) REFERENCES psets(id) ON DELETE RESTRICT
                );
                CREATE INDEX IF NOT EXISTS idx_job_steps_pset_id ON job_steps(pset_id);",
            )
            .map_err(|error| format!("Failed to initialize Job schema: {error}"))?;

        let jobs_schema = connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'jobs'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| format!("Failed to inspect Job schema: {error}"))?
            .unwrap_or_default();
        if jobs_schema.contains("BETWEEN 0 AND 99)") && !jobs_schema.contains("BETWEEN 0 AND 9999)")
        {
            connection
                .execute_batch(
                    "PRAGMA foreign_keys = OFF;
                     BEGIN IMMEDIATE;
                     ALTER TABLE job_steps RENAME TO job_steps_legacy;
                     ALTER TABLE jobs RENAME TO jobs_legacy;
                     CREATE TABLE jobs (
                        id INTEGER PRIMARY KEY CHECK (id BETWEEN 0 AND 9999),
                        name TEXT NOT NULL,
                        forced_order INTEGER NOT NULL,
                        first_tightening_timeout INTEGER NOT NULL,
                        job_timeout INTEGER NOT NULL,
                        batch_count_mode INTEGER NOT NULL,
                        lock_at_job_done INTEGER NOT NULL,
                        use_line_control INTEGER NOT NULL,
                        repeat_job INTEGER NOT NULL,
                        loosening_mode INTEGER NOT NULL,
                        repair_mode INTEGER NOT NULL,
                        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                     );
                     CREATE TABLE job_steps (
                        job_id INTEGER NOT NULL,
                        step_order INTEGER NOT NULL,
                        channel_id INTEGER NOT NULL,
                        pset_id INTEGER NOT NULL,
                        auto_value INTEGER NOT NULL,
                        batch_size INTEGER NOT NULL,
                        PRIMARY KEY (job_id, step_order),
                        FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE,
                        FOREIGN KEY (pset_id) REFERENCES psets(id) ON DELETE RESTRICT
                     );
                     INSERT INTO jobs SELECT * FROM jobs_legacy;
                     INSERT INTO job_steps SELECT * FROM job_steps_legacy;
                     DROP TABLE job_steps_legacy;
                     DROP TABLE jobs_legacy;
                     CREATE INDEX idx_job_steps_pset_id ON job_steps(pset_id);
                     COMMIT;
                     PRAGMA foreign_keys = ON;",
                )
                .map_err(|error| format!("Failed to migrate Job ID width: {error}"))?;
        }
        Ok(())
    }

    fn row_to_job(row: &rusqlite::Row<'_>) -> SqliteResult<Job> {
        Ok(Job {
            id: row.get::<_, i64>(0)? as u32,
            name: row.get(1)?,
            forced_order: row.get::<_, i64>(2)? as u8,
            first_tightening_timeout: row.get::<_, i64>(3)? as u32,
            job_timeout: row.get::<_, i64>(4)? as u32,
            batch_count_mode: row.get::<_, i64>(5)? as u8,
            lock_at_job_done: row.get::<_, i64>(6)? != 0,
            use_line_control: row.get::<_, i64>(7)? != 0,
            repeat_job: row.get::<_, i64>(8)? != 0,
            loosening_mode: row.get::<_, i64>(9)? as u8,
            repair_mode: row.get::<_, i64>(10)? as u8,
            steps: Vec::new(),
        })
    }

    fn load_steps(connection: &rusqlite::Connection, job_id: u32) -> Result<Vec<JobStep>, String> {
        let mut statement = connection
            .prepare(
                "SELECT channel_id, pset_id, auto_value, batch_size
                 FROM job_steps WHERE job_id = ?1 ORDER BY step_order",
            )
            .map_err(|error| format!("Failed to prepare Job step query: {error}"))?;
        let rows = statement
            .query_map(params![job_id as i64], |row| {
                Ok(JobStep {
                    channel_id: row.get::<_, i64>(0)? as u32,
                    pset_id: row.get::<_, i64>(1)? as u32,
                    auto_value: row.get::<_, i64>(2)? != 0,
                    batch_size: row.get::<_, i64>(3)? as u32,
                })
            })
            .map_err(|error| format!("Failed to query Job steps: {error}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Failed to read Job steps: {error}"))
    }

    fn insert_steps(transaction: &rusqlite::Transaction<'_>, job: &Job) -> Result<(), String> {
        for (index, step) in job.steps.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO job_steps
                     (job_id, step_order, channel_id, pset_id, auto_value, batch_size)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        job.id as i64,
                        index as i64,
                        step.channel_id as i64,
                        step.pset_id as i64,
                        step.auto_value as i64,
                        step.batch_size as i64
                    ],
                )
                .map_err(|error| format!("Failed to store Job step {}: {error}", index + 1))?;
        }
        Ok(())
    }
}

impl JobRepository for SqliteJobRepository {
    fn get_all(&self) -> Vec<Job> {
        let connection = match self.pool.get() {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("Failed to get connection: {error}");
                return Vec::new();
            }
        };
        let mut statement = match connection.prepare(
            "SELECT id, name, forced_order, first_tightening_timeout, job_timeout,
                    batch_count_mode, lock_at_job_done, use_line_control, repeat_job,
                    loosening_mode, repair_mode
             FROM jobs ORDER BY id",
        ) {
            Ok(statement) => statement,
            Err(error) => {
                eprintln!("Failed to prepare Job query: {error}");
                return Vec::new();
            }
        };
        let jobs = match statement.query_map([], Self::row_to_job) {
            Ok(rows) => rows.filter_map(Result::ok).collect::<Vec<_>>(),
            Err(error) => {
                eprintln!("Failed to query Jobs: {error}");
                return Vec::new();
            }
        };
        jobs.into_iter()
            .filter_map(|mut job| {
                job.steps = Self::load_steps(&connection, job.id).ok()?;
                Some(job)
            })
            .collect()
    }

    fn get_by_id(&self, id: u32) -> Option<Job> {
        let connection = self.pool.get().ok()?;
        let mut job = connection
            .query_row(
                "SELECT id, name, forced_order, first_tightening_timeout, job_timeout,
                        batch_count_mode, lock_at_job_done, use_line_control, repeat_job,
                        loosening_mode, repair_mode
                 FROM jobs WHERE id = ?1",
                params![id as i64],
                Self::row_to_job,
            )
            .optional()
            .ok()??;
        job.steps = Self::load_steps(&connection, id).ok()?;
        Some(job)
    }

    fn create(&mut self, job: Job) -> Result<Job, String> {
        let mut connection = self
            .pool
            .get()
            .map_err(|error| format!("Failed to get connection: {error}"))?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Failed to start Job transaction: {error}"))?;
        transaction
            .execute(
                "INSERT INTO jobs
                 (id, name, forced_order, first_tightening_timeout, job_timeout,
                  batch_count_mode, lock_at_job_done, use_line_control, repeat_job,
                  loosening_mode, repair_mode)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    job.id as i64,
                    job.name,
                    job.forced_order as i64,
                    job.first_tightening_timeout as i64,
                    job.job_timeout as i64,
                    job.batch_count_mode as i64,
                    job.lock_at_job_done as i64,
                    job.use_line_control as i64,
                    job.repeat_job as i64,
                    job.loosening_mode as i64,
                    job.repair_mode as i64
                ],
            )
            .map_err(|error| {
                if error.to_string().contains("UNIQUE constraint failed") {
                    format!("Job with id {:02} already exists", job.id)
                } else {
                    format!("Failed to create Job: {error}")
                }
            })?;
        Self::insert_steps(&transaction, &job)?;
        transaction
            .commit()
            .map_err(|error| format!("Failed to commit Job transaction: {error}"))?;
        self.get_by_id(job.id)
            .ok_or_else(|| "Failed to retrieve created Job".to_string())
    }

    fn update(&mut self, id: u32, mut job: Job) -> Result<Job, String> {
        job.id = id;
        let mut connection = self
            .pool
            .get()
            .map_err(|error| format!("Failed to get connection: {error}"))?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Failed to start Job transaction: {error}"))?;
        let affected = transaction
            .execute(
                "UPDATE jobs SET
                    name = ?1, forced_order = ?2, first_tightening_timeout = ?3,
                    job_timeout = ?4, batch_count_mode = ?5, lock_at_job_done = ?6,
                    use_line_control = ?7, repeat_job = ?8, loosening_mode = ?9,
                    repair_mode = ?10, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?11",
                params![
                    job.name,
                    job.forced_order as i64,
                    job.first_tightening_timeout as i64,
                    job.job_timeout as i64,
                    job.batch_count_mode as i64,
                    job.lock_at_job_done as i64,
                    job.use_line_control as i64,
                    job.repeat_job as i64,
                    job.loosening_mode as i64,
                    job.repair_mode as i64,
                    id as i64
                ],
            )
            .map_err(|error| format!("Failed to update Job: {error}"))?;
        if affected == 0 {
            return Err(format!("Job with id {:02} not found", id));
        }
        transaction
            .execute(
                "DELETE FROM job_steps WHERE job_id = ?1",
                params![id as i64],
            )
            .map_err(|error| format!("Failed to replace Job steps: {error}"))?;
        Self::insert_steps(&transaction, &job)?;
        transaction
            .commit()
            .map_err(|error| format!("Failed to commit Job transaction: {error}"))?;
        self.get_by_id(id)
            .ok_or_else(|| "Failed to retrieve updated Job".to_string())
    }

    fn delete(&mut self, id: u32) -> Result<(), String> {
        let connection = self
            .pool
            .get()
            .map_err(|error| format!("Failed to get connection: {error}"))?;
        let affected = connection
            .execute("DELETE FROM jobs WHERE id = ?1", params![id as i64])
            .map_err(|error| format!("Failed to delete Job: {error}"))?;
        if affected == 0 {
            Err(format!("Job with id {:02} not found", id))
        } else {
            Ok(())
        }
    }

    fn references_pset(&self, pset_id: u32) -> bool {
        let connection = match self.pool.get() {
            Ok(connection) => connection,
            Err(_) => return false,
        };
        connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM job_steps WHERE pset_id = ?1)",
                params![pset_id as i64],
                |row| row.get::<_, bool>(0),
            )
            .unwrap_or(false)
    }
}

pub type SharedJobRepository = Arc<RwLock<Box<dyn JobRepository>>>;

pub fn create_default_repository() -> SharedJobRepository {
    Arc::new(RwLock::new(Box::new(InMemoryJobRepository::new())))
}

pub fn create_sqlite_repository(db_path: &str) -> Result<SharedJobRepository, String> {
    Ok(Arc::new(RwLock::new(Box::new(SqliteJobRepository::new(
        db_path,
    )?))))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Running,
    Ok,
    Nok,
}

impl JobStatus {
    pub fn protocol_value(self) -> u8 {
        match self {
            Self::Running => 0,
            Self::Ok => 1,
            Self::Nok => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecution {
    pub job: Job,
    pub current_step_index: usize,
    pub step_counter: u32,
    pub total_counter: u32,
    pub total_batch_size: u32,
    pub status: JobStatus,
    pub has_nok: bool,
    pub started_at: DateTime<Utc>,
    pub last_update_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct JobProgress {
    pub job_id: u32,
    pub pset_id: u32,
    pub batch_size: u32,
    pub step_counter: u32,
    pub total_counter: u32,
    pub previous_step_index: usize,
    pub current_step_index: usize,
    pub counted: bool,
    pub step_changed: bool,
    pub completed_status: Option<JobStatus>,
    pub repeated: bool,
}

impl JobExecution {
    pub fn new(job: Job) -> Self {
        let now = Utc::now();
        let total_batch_size = job.total_batch_size();
        Self {
            job,
            current_step_index: 0,
            step_counter: 0,
            total_counter: 0,
            total_batch_size,
            status: JobStatus::Running,
            has_nok: false,
            started_at: now,
            last_update_at: now,
            completed_at: None,
        }
    }

    pub fn current_step(&self) -> &JobStep {
        &self.job.steps[self.current_step_index]
    }

    pub fn is_running(&self) -> bool {
        self.status == JobStatus::Running
    }

    pub fn restart(&mut self) {
        let now = Utc::now();
        self.current_step_index = 0;
        self.step_counter = 0;
        self.total_counter = 0;
        self.status = JobStatus::Running;
        self.has_nok = false;
        self.started_at = now;
        self.last_update_at = now;
        self.completed_at = None;
    }

    pub fn record_tightening(&mut self, ok: bool) -> JobProgress {
        let should_count = ok || self.job.batch_count_mode == 1;
        self.advance(should_count, should_count && !ok)
    }

    pub fn increment(&mut self) -> JobProgress {
        self.advance(true, false)
    }

    fn advance(&mut self, counted: bool, counted_nok: bool) -> JobProgress {
        let previous_step_index = self.current_step_index;
        let step = self.current_step().clone();
        let mut completed_status = None;
        let mut step_changed = false;
        let mut repeated = false;

        if counted && self.is_running() {
            self.step_counter += 1;
            self.total_counter += 1;
            if counted_nok {
                self.has_nok = true;
            }

            if self.step_counter >= step.batch_size {
                if self.current_step_index + 1 < self.job.steps.len() {
                    self.current_step_index += 1;
                    self.step_counter = 0;
                    step_changed = true;
                } else {
                    let final_status = if self.has_nok {
                        JobStatus::Nok
                    } else {
                        JobStatus::Ok
                    };
                    completed_status = Some(final_status);
                    self.status = final_status;
                    self.completed_at = Some(Utc::now());
                    if self.job.repeat_job {
                        self.restart();
                        step_changed = true;
                        repeated = true;
                    }
                }
            }
        }

        self.last_update_at = Utc::now();
        JobProgress {
            job_id: self.job.id,
            pset_id: step.pset_id,
            batch_size: step.batch_size,
            step_counter: if counted {
                if step_changed || completed_status.is_some() {
                    step.batch_size
                } else {
                    self.step_counter
                }
            } else {
                self.step_counter
            },
            total_counter: if repeated {
                self.total_batch_size
            } else {
                self.total_counter
            },
            previous_step_index,
            current_step_index: self.current_step_index,
            counted,
            step_changed,
            completed_status,
            repeated,
        }
    }

    pub fn runtime_state(&self) -> JobRuntimeState {
        JobRuntimeState {
            job_id: self.job.id,
            job_name: self.job.name.clone(),
            status: self.status,
            batch_count_mode: self.job.batch_count_mode,
            current_step: self.current_step_index as u32 + 1,
            total_steps: self.job.steps.len() as u32,
            current_pset_id: self.current_step().pset_id,
            step_progress: self.step_counter,
            step_batch_size: self.current_step().batch_size,
            total_progress: self.total_counter,
            total_batch_size: self.total_batch_size,
            timestamp: self.last_update_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRuntimeState {
    pub job_id: u32,
    pub job_name: String,
    pub status: JobStatus,
    pub batch_count_mode: u8,
    pub current_step: u32,
    pub total_steps: u32,
    pub current_pset_id: u32,
    pub step_progress: u32,
    pub step_batch_size: u32,
    pub total_progress: u32,
    pub total_batch_size: u32,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn example_job() -> Job {
        Job {
            id: 1,
            name: "Example".to_string(),
            forced_order: 1,
            first_tightening_timeout: 0,
            job_timeout: 0,
            batch_count_mode: 0,
            lock_at_job_done: false,
            use_line_control: false,
            repeat_job: false,
            loosening_mode: 0,
            repair_mode: 0,
            steps: vec![
                JobStep {
                    channel_id: 1,
                    pset_id: 1,
                    auto_value: true,
                    batch_size: 2,
                },
                JobStep {
                    channel_id: 1,
                    pset_id: 2,
                    auto_value: true,
                    batch_size: 1,
                },
            ],
        }
    }

    #[test]
    fn validates_revision_one_ranges() {
        assert!(example_job().validate(&[1], &[1, 2]).is_ok());
        let mut invalid = example_job();
        invalid.name = "non-ascii-\u{e1}".to_string();
        assert!(invalid.validate(&[1], &[1, 2]).is_err());
        invalid.name = "line\nbreak".to_string();
        assert!(invalid.validate(&[1], &[1, 2]).is_err());
    }

    #[test]
    fn ok_only_mode_retries_nok() {
        let mut execution = JobExecution::new(example_job());
        let nok = execution.record_tightening(false);
        assert!(!nok.counted);
        assert_eq!(execution.step_counter, 0);
        assert!(!execution.has_nok);
    }

    #[test]
    fn advances_steps_and_completes() {
        let mut execution = JobExecution::new(example_job());
        execution.record_tightening(true);
        let transition = execution.record_tightening(true);
        assert!(transition.step_changed);
        assert_eq!(execution.current_step_index, 1);
        let complete = execution.record_tightening(true);
        assert_eq!(complete.completed_status, Some(JobStatus::Ok));
        assert_eq!(execution.status, JobStatus::Ok);
    }

    #[test]
    fn in_memory_repository_preserves_step_order() {
        let mut repository = InMemoryJobRepository::new();
        let created = repository.create(example_job()).unwrap();
        assert_eq!(created.steps[0].pset_id, 1);
        assert_eq!(created.steps[1].pset_id, 2);
        assert!(repository.references_pset(2));
        assert!(repository.create(example_job()).is_err());
    }

    #[test]
    fn sqlite_repository_round_trips_jobs_transactionally() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("job-repository-{suffix}.db"));
        let path_string = path.to_string_lossy().to_string();
        let _psets = crate::pset::SqlitePsetRepository::new(&path_string).unwrap();
        let mut repository = SqliteJobRepository::new(&path_string).unwrap();

        repository.create(example_job()).unwrap();
        let stored = repository.get_by_id(1).unwrap();
        assert_eq!(stored, example_job());
        assert!(repository.references_pset(2));

        let mut updated = stored;
        updated.name = "Updated".to_string();
        updated.steps.reverse();
        repository.update(1, updated.clone()).unwrap();
        assert_eq!(repository.get_by_id(1).unwrap(), updated);

        repository.delete(1).unwrap();
        assert!(repository.get_by_id(1).is_none());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn batch_count_mode_one_counts_nok_and_marks_job_nok() {
        let mut job = example_job();
        job.batch_count_mode = 1;
        job.steps = vec![JobStep {
            channel_id: 1,
            pset_id: 1,
            auto_value: true,
            batch_size: 1,
        }];
        let mut execution = JobExecution::new(job);
        let progress = execution.record_tightening(false);
        assert!(progress.counted);
        assert_eq!(progress.completed_status, Some(JobStatus::Nok));
    }

    #[test]
    fn repeat_job_restarts_after_completion() {
        let mut job = example_job();
        job.repeat_job = true;
        job.steps = vec![JobStep {
            channel_id: 1,
            pset_id: 1,
            auto_value: true,
            batch_size: 1,
        }];
        let mut execution = JobExecution::new(job);
        let progress = execution.record_tightening(true);
        assert!(progress.repeated);
        assert_eq!(execution.status, JobStatus::Running);
        assert_eq!(execution.total_counter, 0);
    }
}
