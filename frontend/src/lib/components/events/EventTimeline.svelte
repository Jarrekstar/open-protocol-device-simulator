<script lang="ts">
	import type { SimulatorEvent } from '$lib/types';
	import EventDisplay from './EventDisplay.svelte';
	import Badge from '../ui/Badge.svelte';

	interface Props {
		/** Events to display */
		events: SimulatorEvent[];
		/** Maximum number of events to show (default: all) */
		limit?: number;
		/** Show event numbers */
		showNumbers?: boolean;
		/** Empty state message */
		emptyMessage?: string;
	}

	let {
		events,
		limit,
		showNumbers = false,
		emptyMessage = 'No events yet'
	}: Props = $props();

	const displayEvents = $derived(limit ? events.slice(0, limit) : events);
</script>

{#if displayEvents.length > 0}
	<ol class="relative border-l border-surface-200-700-token pl-6">
		{#each displayEvents as event, index}
			<li class="mb-6 last:mb-0">
				<span
					class="absolute -left-3 mt-2 h-3 w-3 rounded-full border border-surface-100-800-token bg-primary-500"
					aria-hidden="true"
				/>
				<div class="flex items-center justify-between">
					<Badge variant="soft">{event.type}</Badge>
					{#if showNumbers}
						<span class="text-xs uppercase tracking-wide text-surface-600-300-token"
							>#{index + 1}</span
						>
					{/if}
				</div>
				<div class="mt-3">
					<EventDisplay {event} />
				</div>
			</li>
		{/each}
	</ol>
{:else}
	<p class="opacity-70">{emptyMessage}</p>
{/if}
