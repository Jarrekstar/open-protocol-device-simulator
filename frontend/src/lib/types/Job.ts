export type JobStatus = 'running' | 'ok' | 'nok';

export interface JobStep {
	channel_id: number;
	pset_id: number;
	auto_value: boolean;
	batch_size: number;
}

export interface Job {
	id: number;
	name: string;
	forced_order: number;
	first_tightening_timeout: number;
	job_timeout: number;
	batch_count_mode: number;
	lock_at_job_done: boolean;
	use_line_control: boolean;
	repeat_job: boolean;
	loosening_mode: number;
	repair_mode: number;
	steps: JobStep[];
}

export interface JobRuntimeState {
	job_id: number;
	job_name: string;
	status: JobStatus;
	batch_count_mode: number;
	current_step: number;
	total_steps: number;
	current_pset_id: number;
	step_progress: number;
	step_batch_size: number;
	total_progress: number;
	total_batch_size: number;
	timestamp: string;
}
