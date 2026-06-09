import type { TighteningResult } from './TighteningResult';
import type { MultiSpindleResult, MultiSpindleStatus } from './MultiSpindle';
import type { JobRuntimeState } from './Job';

export type SimulatorEvent =
	| { type: 'TighteningCompleted'; result: TighteningResult }
	| { type: 'PsetChanged'; pset_id: number; pset_name: string }
	| { type: 'ToolStateChanged'; enabled: boolean }
	| { type: 'BatchCompleted'; total: number }
	| { type: 'VehicleIdChanged'; vin: string }
	| { type: 'MultiSpindleStatusCompleted'; status: MultiSpindleStatus }
	| {
			type: 'MultiSpindleResultCompleted';
			result: MultiSpindleResult;
			job_id: number;
			pset_id: number;
			batch_size: number;
			batch_counter: number;
			batch_status: number;
	  }
	| { type: 'AutoTighteningProgress'; counter: number; target_size: number; running: boolean }
	| { type: 'JobSelected'; state: JobRuntimeState }
	| { type: 'JobProgress'; state: JobRuntimeState }
	| { type: 'JobStepChanged'; state: JobRuntimeState; previous_step: number }
	| { type: 'JobRestarted'; state: JobRuntimeState }
	| { type: 'JobCompleted'; state: JobRuntimeState };
