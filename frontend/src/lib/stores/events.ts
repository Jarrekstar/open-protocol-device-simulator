import { writable, derived } from 'svelte/store';
import type { SimulatorEvent } from '$lib/types';

export const events = writable<SimulatorEvent[]>([]);
export const eventFilter = writable<string>('all');
export const eventSearchQuery = writable<string>('');

export function addEvent(event: SimulatorEvent) {
	events.update((list) => {
		const updated = [event, ...list];
		// Keep last 1000 events
		return updated.slice(0, 1000);
	});
}

export function clearEvents() {
	events.set([]);
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

		// Filter by search query
		if ($query.trim()) {
			const query = $query.toLowerCase();
			filtered = filtered.filter((e) => JSON.stringify(e).toLowerCase().includes(query));
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
