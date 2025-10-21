export interface TighteningResult {
	cell_id: number;
	channel_id: number;
	controller_name: string;
	vin_number: string | null;
	job_id: number;
	pset_id: number;
	batch_size: number;
	batch_counter: number;
	tightening_status: boolean;
	torque_status: boolean;
	angle_status: boolean;
	torque_min: number;
	torque_max: number;
	torque_target: number;
	torque: number;
	angle_min: number;
	angle_max: number;
	angle_target: number;
	angle: number;
	timestamp: string;
	last_pset_change: string | null;
	batch_status: boolean | null;
	tightening_id: number | null;
}
