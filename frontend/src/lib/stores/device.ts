import { writable, derived } from 'svelte/store';
import type { DeviceState } from '$lib/types';

export const deviceState = writable<DeviceState | null>(null);

// Derived stores for specific properties
export const isMultiSpindleEnabled = derived(
	deviceState,
	($state) => $state?.multi_spindle_config.enabled ?? false
);

export const currentToolState = derived(deviceState, ($state) => $state?.tool_state ?? 'Unknown');

export const batchProgress = derived(deviceState, ($state) => {
	if (!$state) return null;

	// Get batch info from tightening tracker (via latest tightening event)
	// This will be updated by events
	return null; // Will be populated by tightening events
});
