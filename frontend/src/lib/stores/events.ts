import { writable, derived } from 'svelte/store';
import type { SimulatorEvent } from '$lib/types';
import { EVENTS } from '$lib/config/constants';

export const events = writable<SimulatorEvent[]>([]);
export const eventFilter = writable<string>('all');
export const eventSearchQuery = writable<string>('');

export function addEvent(event: SimulatorEvent) {
	events.update((list) => {
		const updated = [event, ...list];
		// Keep last N events as configured
		return updated.slice(0, EVENTS.MAX_EVENTS);
	});
}

export function clearEvents() {
	events.set([]);
}

/**
 * Checks if an event matches a search query
 * Searches specific indexed fields instead of stringifying entire object
 * @param event - Event to search
 * @param query - Lowercase search query
 * @returns True if event matches query
 */
function matchesSearchQuery(event: SimulatorEvent, query: string): boolean {
	// Search event type
	if (event.type.toLowerCase().includes(query)) return true;

	// Search type-specific fields
	switch (event.type) {
		case 'PsetChanged':
			return (
				event.pset_name.toLowerCase().includes(query) ||
				event.pset_id.toString().includes(query)
			);
		case 'VehicleIdChanged':
			return event.vin.toLowerCase().includes(query);
		case 'BatchCompleted':
			return event.total.toString().includes(query);
		case 'TighteningCompleted':
			return (
				event.result.tightening_status.toString().includes(query) ||
				event.result.torque.toString().includes(query) ||
				event.result.angle.toString().includes(query)
			);
		case 'MultiSpindleResultCompleted':
			return (
				event.result.overall_status.toString().includes(query) ||
				event.result.spindle_count.toString().includes(query)
			);
		case 'ToolStateChanged':
			return event.enabled.toString().includes(query);
		default:
			return false;
	}
}

// Filtered events based on filter and search
export const filteredEvents = derived(
	[events, eventFilter, eventSearchQuery],
	([$events, $filter, $query]) => {
		let filtered = $events;

		// Filter by type
		if ($filter !== 'all') {
			filtered = filtered.filter((e) => e.type === $filter);
		}

		// Filter by search query using indexed field search
		if ($query.trim()) {
			const query = $query.toLowerCase();
			filtered = filtered.filter((e) => matchesSearchQuery(e, query));
		}

		return filtered;
	}
);

// Event type counts for badges
export const eventCounts = derived(events, ($events) => {
	const counts: Record<string, number> = {};
	$events.forEach((e) => {
		counts[e.type] = (counts[e.type] || 0) + 1;
	});
	return counts;
});
