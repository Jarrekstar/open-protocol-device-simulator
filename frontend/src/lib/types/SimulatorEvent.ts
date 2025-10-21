import type { TighteningResult } from './TighteningResult';
import type { MultiSpindleResult, MultiSpindleStatus } from './MultiSpindle';

export type SimulatorEvent =
	| { type: 'TighteningCompleted'; result: TighteningResult }
	| { type: 'PsetChanged'; pset_id: number; pset_name: string }
	| { type: 'ToolStateChanged'; enabled: boolean }
	| { type: 'BatchCompleted'; total: number }
	| { type: 'VehicleIdChanged'; vin: string }
	| { type: 'MultiSpindleStatusCompleted'; status: MultiSpindleStatus }
	| { type: 'MultiSpindleResultCompleted'; result: MultiSpindleResult }
	| { type: 'AutoTighteningProgress'; counter: number; target_size: number; running: boolean };
