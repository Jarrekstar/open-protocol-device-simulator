// Re-export all types
import type { OperationMode } from './DeviceState';

export type {
	DeviceState,
	MultiSpindleConfig,
	FailureConfig,
	OperationMode
} from './DeviceState';
export type { TighteningResult } from './TighteningResult';
export type { SpindleResult, MultiSpindleResult, MultiSpindleStatus } from './MultiSpindle';
export type { SimulatorEvent } from './SimulatorEvent';
export type { Pset } from './Pset';
export type { Job, JobStep, JobStatus, JobRuntimeState } from './Job';
export type {
	MidFamilyDefinition,
	ProtocolProfile,
	ProtocolSampleData,
	RevisionFeature,
	RevisionPolicy,
	RevisionSelection
} from './Protocol';

// API request/response types
export interface AutoTighteningRequest {
	interval_ms?: number;
	duration_ms?: number;
	failure_rate?: number;
}

export interface OperationModeRequest {
	mode: OperationMode;
}

export interface OperationModeResponse {
	success: boolean;
	message: string;
	mode: OperationMode;
	batch_size: number;
	current_job_id: number | null;
	auto_tightening_stopped: boolean;
}

export interface MultiSpindleConfigRequest {
	enabled: boolean;
	spindle_count?: number;
	sync_id?: number;
}

export interface TighteningRequest {
	torque?: number;
	angle?: number;
	ok?: boolean;
}

export interface FailureConfigRequest {
	connection_health?: number;
	enabled?: boolean;
	packet_loss_rate?: number;
	delay_min_ms?: number;
	delay_max_ms?: number;
	corruption_rate?: number;
	force_disconnect_rate?: number;
}
