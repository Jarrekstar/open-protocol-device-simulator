import { writable, derived } from 'svelte/store';
import type { TighteningResult } from '$lib/types';

export const latestTighteningResult = writable<TighteningResult | null>(null);
export const tighteningHistory = writable<TighteningResult[]>([]);

export interface AutoTighteningProgress {
	counter: number;
	target_size: number;
	running: boolean;
}

export const autoTighteningProgress = writable<AutoTighteningProgress>({
	counter: 0,
	target_size: 0,
	running: false
});

export function addTighteningResult(result: TighteningResult) {
	latestTighteningResult.set(result);
	tighteningHistory.update((list) => {
		const updated = [result, ...list];
		return updated.slice(0, 100); // Keep last 100
	});
}

// Statistics derived from history
export const tighteningStats = derived(tighteningHistory, ($history) => {
	if ($history.length === 0) {
		return {
			total: 0,
			successful: 0,
			successRate: 0,
			avgTorque: 0,
			avgAngle: 0
		};
	}

	const successful = $history.filter((r) => r.tightening_status).length;
	const avgTorque = $history.reduce((sum, r) => sum + r.torque, 0) / $history.length;
	const avgAngle = $history.reduce((sum, r) => sum + r.angle, 0) / $history.length;

	return {
		total: $history.length,
		successful,
		successRate: (successful / $history.length) * 100,
		avgTorque,
		avgAngle
	};
});
