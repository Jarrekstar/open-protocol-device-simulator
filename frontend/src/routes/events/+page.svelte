<script lang="ts">
	import { filteredEvents, eventFilter, eventSearchQuery, clearEvents, eventCounts } from '$lib/stores/events';

	const eventTypes = [
		'all',
		'TighteningCompleted',
		'MultiSpindleResultCompleted',
		'MultiSpindleStatusCompleted',
		'BatchCompleted',
		'ToolStateChanged',
		'PsetChanged',
		'VehicleIdChanged'
	];
</script>

<svelte:head>
	<title>Events - Device Simulator</title>
</svelte:head>

<div class="flex justify-between items-center mb-6">
	<h1 class="h1">Event Log</h1>
	<button class="btn variant-filled-error" on:click={clearEvents}>
		Clear Events
	</button>
</div>

<!-- Filters -->
<div class="card p-4 mb-4">
	<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
		<label class="label">
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

		<label class="label">
			<span>Search</span>
			<input
				type="search"
				class="input"
				placeholder="Search events..."
				bind:value={$eventSearchQuery}
			/>
		</label>
	</div>
</div>

<!-- Event List -->
<div class="space-y-2">
	{#if $filteredEvents.length > 0}
		{#each $filteredEvents as event, i}
			<div class="card p-4">
				<div class="flex justify-between items-start">
					<div class="flex-1">
						<div class="flex items-center gap-2 mb-2">
							<span class="badge variant-filled-primary">{event.type}</span>
							<span class="text-sm text-surface-500">#{i + 1}</span>
						</div>

						{#if event.type === 'TighteningCompleted'}
							<dl class="grid grid-cols-2 md:grid-cols-4 gap-2 text-sm">
								<div>
									<dt class="font-semibold">Status:</dt>
									<dd>
										<span
											class="badge badge-sm"
											class:variant-filled-success={event.result.tightening_status}
											class:variant-filled-error={!event.result.tightening_status}
										>
											{event.result.tightening_status ? 'OK' : 'NOK'}
										</span>
									</dd>
								</div>
								<div>
									<dt class="font-semibold">Torque:</dt>
									<dd>{event.result.torque.toFixed(2)} Nm</dd>
								</div>
								<div>
									<dt class="font-semibold">Angle:</dt>
									<dd>{event.result.angle.toFixed(1)}°</dd>
								</div>
								<div>
									<dt class="font-semibold">Batch:</dt>
									<dd>{event.result.batch_counter} / {event.result.batch_size}</dd>
								</div>
							</dl>
						{:else if event.type === 'MultiSpindleResultCompleted'}
							<dl class="grid grid-cols-2 md:grid-cols-4 gap-2 text-sm">
								<div>
									<dt class="font-semibold">Status:</dt>
									<dd>
										<span
											class="badge badge-sm"
											class:variant-filled-success={event.result.overall_status === 0}
											class:variant-filled-error={event.result.overall_status === 1}
										>
											{event.result.overall_status === 0 ? 'OK' : 'NOK'}
										</span>
									</dd>
								</div>
								<div>
									<dt class="font-semibold">Spindles:</dt>
									<dd>{event.result.spindle_count}</dd>
								</div>
								<div>
									<dt class="font-semibold">Sync ID:</dt>
									<dd>{event.result.sync_id}</dd>
								</div>
								<div>
									<dt class="font-semibold">Result ID:</dt>
									<dd>{event.result.result_id}</dd>
								</div>
							</dl>
						{:else if event.type === 'BatchCompleted'}
							<p class="text-sm">Total: {event.total}</p>
						{:else if event.type === 'ToolStateChanged'}
							<p class="text-sm">
								Tool {event.enabled ? 'enabled' : 'disabled'}
							</p>
						{:else if event.type === 'PsetChanged'}
							<p class="text-sm">PSet: {event.pset_name} (ID: {event.pset_id})</p>
						{:else if event.type === 'VehicleIdChanged'}
							<p class="text-sm">VIN: {event.vin}</p>
						{:else if event.type === 'MultiSpindleStatusCompleted'}
							<p class="text-sm">
								Status: {event.status.status === 0 ? 'Waiting' : event.status.status === 1 ? 'Running' : 'Completed'}
								(Sync ID: {event.status.sync_id}, Spindles: {event.status.spindle_count})
							</p>
						{/if}
					</div>
				</div>
			</div>
		{/each}
	{:else}
		<div class="card p-8 text-center">
			<p class="text-surface-500">No events match your filters</p>
		</div>
	{/if}
</div>

{#if $filteredEvents.length > 0}
	<div class="text-center text-sm text-surface-500 mt-4">
		Showing {$filteredEvents.length} event(s)
	</div>
{/if}
