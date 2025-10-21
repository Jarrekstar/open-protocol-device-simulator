// Re-export all types
export type { DeviceState, MultiSpindleConfig } from './DeviceState';
export type { TighteningResult } from './TighteningResult';
export type { SpindleResult, MultiSpindleResult, MultiSpindleStatus } from './MultiSpindle';
export type { SimulatorEvent } from './SimulatorEvent';
export type { Pset } from './Pset';

// API request/response types
export interface AutoTighteningRequest {
	interval_ms?: number;
	duration_ms?: number;
	failure_rate?: number;
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
