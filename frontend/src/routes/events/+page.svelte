<script lang="ts">
	import {
		filteredEvents,
		eventFilter,
		eventSearchQuery,
		clearEvents,
		eventCounts
	} from '$lib/stores/events';
	import { Button, Badge } from '$lib/components/ui';
	import DetailedEventCard from '$lib/components/events/DetailedEventCard.svelte';
	import DataCard from '$lib/components/ui/DataCard.svelte';
	import ViewModeToggle from '$lib/components/ui/ViewModeToggle.svelte';
	import ChipFilter from '$lib/components/ui/ChipFilter.svelte';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import { formatTorque, formatAngle, formatBatchCounter } from '$lib/utils';

	let viewMode = $state('table');

	const eventTypes = [
		{ id: 'all', label: 'All Events', icon: '📋' },
		{ id: 'TighteningCompleted', label: 'Tightening', icon: '⚙️' },
		{ id: 'MultiSpindleResultCompleted', label: 'Multi-Spindle', icon: '🔧' },
		{ id: 'BatchCompleted', label: 'Batch', icon: '📦' },
		{ id: 'ToolStateChanged', label: 'Tool State', icon: '🔄' },
		{ id: 'PsetChanged', label: 'PSET', icon: '⚡' },
		{ id: 'VehicleIdChanged', label: 'Vehicle ID', icon: '🚗' },
		{ id: 'AutoTighteningProgress', label: 'Auto Progress', icon: '⏱️' }
	];

	const viewModes = [
		{ id: 'table', label: 'Table', icon: '📊' },
		{ id: 'detailed', label: 'Detailed', icon: '📝' },
		{ id: 'compact', label: 'Compact', icon: '📋' }
	];

	function handleFilterToggle(filterId: string) {
		if (filterId === 'all') {
			$eventFilter = 'all';
		} else {
			$eventFilter = $eventFilter === filterId ? 'all' : filterId;
		}
	}

	function handleClearAll() {
		$eventFilter = 'all';
		$eventSearchQuery = '';
	}
</script>

<svelte:head>
	<title>Events - Device Simulator</title>
</svelte:head>

<div class="space-y-6 animate-fade-in">
	<!-- Header -->
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-3xl font-semibold text-surface-900 dark:text-surface-100">Event Log</h1>
			<p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
				Real-time simulator events and activity history
			</p>
		</div>
		<div class="flex items-center gap-3">
			<ViewModeToggle modes={viewModes} bind:activeMode={viewMode} onChange={() => {}} />
			<Button variant="filled-error" onclick={clearEvents}>🗑️ Clear All</Button>
		</div>
	</div>

	<!-- Filters & Search -->
	<DataCard padding="md">
		<div class="space-y-4">
			<!-- Search Bar -->
			<div class="relative">
				<svg
					class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-surface-400"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
					/>
				</svg>
				<input
					type="search"
					class="input !pl-11 w-full"
					placeholder="Search events by payload, ID, or data..."
					bind:value={$eventSearchQuery}
				/>
			</div>

			<!-- Filter Chips -->
			<div>
				<div class="flex items-center justify-between mb-3">
					<span class="text-sm font-medium text-surface-700 dark:text-surface-300">
						Event Types
					</span>
					{#if $eventFilter !== 'all' || $eventSearchQuery}
						<button
							type="button"
							class="text-xs text-surface-600 dark:text-surface-400 hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
							onclick={handleClearAll}
						>
							Clear filters
						</button>
					{/if}
				</div>
				<div class="flex flex-wrap gap-2">
					{#each eventTypes as type}
						<ChipFilter
							label="{type.icon} {type.label}{type.id !== 'all' && $eventCounts[type.id]
								? ` (${$eventCounts[type.id]})`
								: ''}"
							value={type.id}
							active={$eventFilter === type.id}
							onClick={handleFilterToggle}
							removable={false}
							variant={$eventFilter === type.id ? 'primary' : 'default'}
						/>
					{/each}
				</div>
			</div>
		</div>
	</DataCard>

	<!-- Event List -->
	<div class="space-y-4">
		{#if $filteredEvents.length > 0}
			<!-- Event Count Header -->
			<div class="flex items-center justify-between">
				<p class="text-sm text-surface-600 dark:text-surface-400">
					Showing <span class="font-semibold text-surface-900 dark:text-surface-100"
						>{$filteredEvents.length}</span
					>
					{$filteredEvents.length === 1 ? 'event' : 'events'}
				</p>
			</div>

			<!-- Events Container -->
			{#if viewMode === 'table'}
				<!-- Table View -->
				<div class="card overflow-hidden">
					<div class="overflow-x-auto">
						<table class="w-full">
							<thead class="bg-surface-100 dark:bg-surface-300 border-b border-surface-200 dark:border-surface-700">
								<tr>
									<th class="px-4 py-3 text-left text-xs font-semibold text-surface-400 dark:text-surface-300 uppercase tracking-wider">#</th>
									<th class="px-4 py-3 text-left text-xs font-semibold text-surface-400 dark:text-surface-300 uppercase tracking-wider">Type</th>
									<th class="px-4 py-3 text-left text-xs font-semibold text-surface-400 dark:text-surface-300 uppercase tracking-wider">Status</th>
									<th class="px-4 py-3 text-left text-xs font-semibold text-surface-400 dark:text-surface-300 uppercase tracking-wider">Details</th>
								</tr>
							</thead>
							<tbody class="divide-y divide-surface-200 dark:divide-surface-700">
								{#each $filteredEvents as event, i}
									<tr class="hover:bg-surface-50 dark:hover:bg-surface-800/50 transition-colors animate-slide-up" style="animation-delay: {Math.min(i, 20) * 0.02}s">
										<td class="px-4 py-3 text-sm text-surface-600 dark:text-surface-400">{i + 1}</td>
										<td class="px-4 py-3">
											<Badge variant="filled-primary">{event.type}</Badge>
										</td>
										<td class="px-4 py-3">
											{#if event.type === 'TighteningCompleted'}
												<Badge variant={event.result.tightening_status ? 'filled-success' : 'filled-error'}>
													{event.result.tightening_status ? 'OK' : 'NOK'}
												</Badge>
											{:else if event.type === 'MultiSpindleResultCompleted'}
												<Badge variant={event.result.overall_status === 0 ? 'filled-success' : 'filled-error'}>
													{event.result.overall_status === 0 ? 'OK' : 'NOK'}
												</Badge>
											{:else if event.type === 'ToolStateChanged'}
												<Badge variant={event.enabled ? 'filled-success' : 'filled-error'}>
													{event.enabled ? 'Enabled' : 'Disabled'}
												</Badge>
											{:else if event.type === 'AutoTighteningProgress'}
												<Badge variant={event.running ? 'filled-primary' : 'soft'}>
													{event.running ? 'Running' : 'Stopped'}
												</Badge>
											{:else}
												<span class="text-sm text-surface-500 dark:text-surface-400">—</span>
											{/if}
										</td>
										<td class="px-4 py-3">
											{#if event.type === 'TighteningCompleted'}
												<div class="flex gap-4 text-sm">
													<span><span class="text-surface-600 dark:text-surface-400">Torque:</span> <span class="font-semibold">{formatTorque(event.result.torque)}</span></span>
													<span><span class="text-surface-600 dark:text-surface-400">Angle:</span> <span class="font-semibold">{formatAngle(event.result.angle)}</span></span>
													<span><span class="text-surface-600 dark:text-surface-400">Batch:</span> <span class="font-semibold">{formatBatchCounter(event.result.batch_counter, event.result.batch_size)}</span></span>
												</div>
											{:else if event.type === 'MultiSpindleResultCompleted'}
												<div class="flex gap-4 text-sm">
													<span><span class="text-surface-600 dark:text-surface-400">Spindles:</span> <span class="font-semibold">{event.result.spindle_count}</span></span>
													<span><span class="text-surface-600 dark:text-surface-400">Sync ID:</span> <span class="font-semibold">{event.result.sync_id}</span></span>
												</div>
											{:else if event.type === 'BatchCompleted'}
												<span class="text-sm"><span class="text-surface-600 dark:text-surface-400">Total:</span> <span class="font-semibold">{event.total}</span></span>
											{:else if event.type === 'PsetChanged'}
												<span class="text-sm"><span class="text-surface-600 dark:text-surface-400">PSET:</span> <span class="font-semibold">{event.pset_name} (ID {event.pset_id})</span></span>
											{:else if event.type === 'VehicleIdChanged'}
												<span class="text-sm"><span class="text-surface-600 dark:text-surface-400">VIN:</span> <span class="font-semibold">{event.vin}</span></span>
											{:else if event.type === 'AutoTighteningProgress'}
												<span class="text-sm"><span class="text-surface-600 dark:text-surface-400">Progress:</span> <span class="font-semibold">{formatBatchCounter(event.counter, event.target_size)}</span></span>
											{:else if event.type === 'MultiSpindleStatusCompleted'}
												<div class="flex gap-4 text-sm">
													<span><span class="text-surface-600 dark:text-surface-400">Status:</span> <span class="font-semibold">{event.status.status === 0 ? 'Waiting' : event.status.status === 1 ? 'Running' : 'Completed'}</span></span>
													<span><span class="text-surface-600 dark:text-surface-400">Sync ID:</span> <span class="font-semibold">{event.status.sync_id}</span></span>
												</div>
											{/if}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				</div>
			{:else}
				<div class="space-y-3">
					{#each $filteredEvents as event, i}
						{#if viewMode === 'detailed'}
							<div class="animate-slide-up" style="animation-delay: {Math.min(i, 20) * 0.02}s">
								<DetailedEventCard {event} index={i} />
							</div>
						{:else}
							<!-- Compact View -->
							<div
								class="card p-4 hover-lift flex items-center justify-between gap-4 animate-slide-up"
								style="animation-delay: {Math.min(i, 20) * 0.02}s"
							>
								<div class="flex items-center gap-3 flex-1 min-w-0">
									<div
										class="flex-shrink-0 w-8 h-8 rounded-full bg-primary-100 dark:bg-primary-900 flex items-center justify-center text-xs font-semibold text-primary-700 dark:text-primary-300"
									>
										{i + 1}
									</div>
									<div class="flex-1 min-w-0">
										<span
											class="font-medium text-surface-900 dark:text-surface-100 truncate"
										>
											{event.type}
										</span>
									</div>
								</div>
							</div>
						{/if}
					{/each}
				</div>
			{/if}
		{:else}
			<EmptyState
				title="No Events Found"
				description={$eventSearchQuery
					? `No events match your search "${$eventSearchQuery}"`
					: $eventFilter !== 'all'
						? `No events of type "${$eventFilter}"`
						: 'No events have been recorded yet'}
				icon="🔍"
			>
				{#snippet action()}
					{#if $eventFilter !== 'all' || $eventSearchQuery}
						<button class="btn variant-filled-primary" onclick={handleClearAll}>
							Clear Filters
						</button>
					{:else}
						<a href="/control" class="btn variant-filled-primary">
							Simulate Tightening
						</a>
					{/if}
				{/snippet}
			</EmptyState>
		{/if}
	</div>
</div>
