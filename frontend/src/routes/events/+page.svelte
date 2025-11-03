<script lang="ts">
	import { filteredEvents, eventFilter, eventSearchQuery, clearEvents, eventCounts } from '$lib/stores/events';
	import { Card, Button } from '$lib/components/ui';
	import DetailedEventCard from '$lib/components/events/DetailedEventCard.svelte';

	const eventTypes = [
		'all',
		'TighteningCompleted',
		'MultiSpindleResultCompleted',
		'MultiSpindleStatusCompleted',
		'BatchCompleted',
		'ToolStateChanged',
		'PsetChanged',
		'VehicleIdChanged',
		'AutoTighteningProgress'
	];
</script>

<svelte:head>
	<title>Events - Device Simulator</title>
</svelte:head>

<div class="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
	<h1 class="h1">Event Log</h1>
	<Button variant="filled-error" onclick={clearEvents}>
		Clear Events
	</Button>
</div>

<div class="lg:grid lg:grid-cols-[260px_1fr] lg:items-start lg:gap-6">
	<aside class="space-y-4 lg:sticky lg:top-6">
		<Card padding="sm" spacing="sm">
			<div>
				<h2 class="h3">Filters</h2>
				<p class="text-sm opacity-70">Narrow events by type or payload.</p>
			</div>

			<label class="label text-surface-600-300-token">
				<span>Filter by Type</span>
				<select class="select" bind:value={$eventFilter}>
					{#each eventTypes as type}
						<option value={type}>
							{type}
							{#if type !== 'all' && $eventCounts[type]}
								({$eventCounts[type]})
							{/if}
						</option>
					{/each}
				</select>
			</label>

			<label class="label text-surface-600-300-token">
				<span>Search</span>
				<input
					type="search"
					class="input"
					placeholder="Search events..."
					bind:value={$eventSearchQuery}
				/>
			</label>
		</Card>
	</aside>

	<section class="space-y-4">
		{#if $filteredEvents.length > 0}
			<div
				class="rounded-xl border border-surface-200-700-token bg-surface-100-800-token overflow-hidden"
			>
				{#each $filteredEvents as event, i}
					<DetailedEventCard {event} index={i} />
				{/each}
			</div>

			<div class="text-sm opacity-70">
				Showing {$filteredEvents.length} event(s)
			</div>
		{:else}
			<Card padding="lg" class="text-center">
				<p class="opacity-70">No events match your filters</p>
			</Card>
		{/if}
	</section>
</div>
