export type RevisionPolicy = 'exact' | 'any_implemented';

export interface RevisionSelection {
	enabled: boolean;
	policy: RevisionPolicy;
	revision: number;
}

export interface RevisionFeature {
	revision: number;
	summary: string;
}

export interface MidFamilyDefinition {
	id: string;
	name: string;
	mids: number[];
	supported_revisions: number[];
	implemented_revisions: number[];
	default_revision: number;
	features: RevisionFeature[];
}

export interface ProtocolSampleData {
	open_protocol_version: string;
	controller_software_version: string;
	tool_software_version: string;
	rbu_type: string;
	controller_serial_number: string;
	system_type: number;
	system_subtype: number;
	supports_sequence_number: boolean;
	supports_message_linking: boolean;
	station_id: number;
	identifier_part_1: string;
	identifier_part_2: string;
	identifier_part_3: string;
	identifier_part_4: string;
	job_sequence_number: number;
	job_tightening_status: number;
	tool_serial_number: string;
	tightening_strategy: number;
	tightening_strategy_options: number;
	tightening_error_status: number;
	tightening_error_status_2: number;
	rundown_angle_min: number;
	rundown_angle_max: number;
	rundown_angle: number;
	current_monitoring_min: number;
	current_monitoring_max: number;
	current_monitoring_value: number;
	self_tap_min: number;
	self_tap_max: number;
	self_tap_torque: number;
	prevail_torque_min: number;
	prevail_torque_max: number;
	prevail_torque: number;
	prevail_torque_compensate: number;
	torque_unit: number;
	tightening_result_type: number;
	customer_tightening_error_code: string;
	compensated_angle: number;
	final_angle_decimal: number;
	multistage_count: number;
	multistage_torque: number;
	multistage_angle: number;
	multi_spindle_data_number: number;
	multi_spindle_send_only_new: boolean;
}

export interface ProtocolProfile {
	version: number;
	families: Record<string, RevisionSelection>;
	samples: ProtocolSampleData;
}
