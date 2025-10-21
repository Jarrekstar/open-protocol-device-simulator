export interface Pset {
	id: number;
	name: string;
	torque_min: number;
	torque_max: number;
	angle_min: number;
	angle_max: number;
	description?: string;
}
