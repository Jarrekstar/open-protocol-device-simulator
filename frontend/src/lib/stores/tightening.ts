import { writable, derived } from 'svelte/store';
import type { TighteningResult } from '$lib/types';
import { EVENTS, UI } from '$lib/config/constants';

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
		return updated.slice(0, EVENTS.MAX_TIGHTENING_HISTORY);
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

// Sparkline data derived from history (chronological order)
export const torqueSparkline = derived(tighteningHistory, ($history) => {
	// Take last N results and reverse to show chronological order (oldest to newest)
	return $history
		.slice(0, UI.SPARKLINE_DATA_POINTS)
		.reverse()
		.map((r) => r.torque);
});

export const angleSparkline = derived(tighteningHistory, ($history) => {
	return $history
		.slice(0, UI.SPARKLINE_DATA_POINTS)
		.reverse()
		.map((r) => r.angle);
});

export const successRateSparkline = derived(tighteningHistory, ($history) => {
	if ($history.length === 0) return [];

	// Calculate rolling success rate for last N results
	const results = $history.slice(0, UI.SPARKLINE_DATA_POINTS).reverse();
	const rates: number[] = [];
	let successCount = 0;

	results.forEach((result, index) => {
		if (result.tightening_status) {
			successCount++;
		}
		const rate = (successCount / (index + 1)) * 100;
		rates.push(rate);
	});

	return rates;
});
