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
	device_fsm_state: string;
}

export interface MultiSpindleConfig {
	enabled: boolean;
	spindle_count: number;
	sync_id: number;
}
