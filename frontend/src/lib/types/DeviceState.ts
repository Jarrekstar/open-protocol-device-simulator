export interface DeviceState {
	cell_id: number;
	channel_id: number;
	controller_name: string;
	tool_enabled: boolean;
	tool_state: string;
	vehicle_id_number: string | null;
	current_job_id: number | null;
	current_pset_id: number | null;
	current_pset_name: string | null;
	multi_spindle_config: MultiSpindleConfig;
	failure_config: FailureConfig;
	device_fsm_state: string;
}

export interface MultiSpindleConfig {
	enabled: boolean;
	spindle_count: number;
	sync_id: number;
}

export interface FailureConfig {
	enabled: boolean;
	connection_health: number;
	packet_loss_rate: number;
	delay_min_ms: number;
	delay_max_ms: number;
	corruption_rate: number;
	force_disconnect_rate: number;
}
