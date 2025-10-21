export interface SpindleResult {
	spindle_id: number;
	channel_id: number;
	torque: number;
	angle: number;
	torque_status: number;
	angle_status: number;
}

export interface MultiSpindleResult {
	result_id: number;
	sync_id: number;
	timestamp: string;
	overall_status: number;
	spindle_count: number;
	spindle_results: SpindleResult[];
}

export interface MultiSpindleStatus {
	sync_id: number;
	status: number; // 0=Waiting, 1=Running, 2=Completed
	spindle_count: number;
	timestamp: string;
}
