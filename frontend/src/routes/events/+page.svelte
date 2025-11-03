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

<div class="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
	<h1 class="h1">Event Log</h1>
	<button class="btn variant-ghost-surface btn-error" on:click={clearEvents}>
		Clear Events
	</button>
</div>

<div class="lg:grid lg:grid-cols-[260px_1fr] lg:items-start lg:gap-6">
	<aside class="space-y-4 lg:sticky lg:top-6">
		<div class="card p-5 space-y-4">
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
		</div>
	</aside>

	<section class="space-y-4">
		{#if $filteredEvents.length > 0}
			<div class="rounded-xl border border-surface-200-700-token bg-surface-100-800-token overflow-hidden">
				{#each $filteredEvents as event, i}
					<article class="border-b border-surface-200-700-token px-5 py-4 even:bg-surface-100-800-token last:border-b-0">
						<header class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
							<div class="flex items-center gap-3">
								<span class="badge variant-filled-primary">{event.type}</span>
								<span class="text-xs uppercase tracking-wide text-surface-600-300-token">Event #{i + 1}</span>
							</div>
						</header>

						{#if event.type === 'TighteningCompleted'}
							<dl class="mt-3 grid grid-cols-2 gap-3 text-sm md:grid-cols-4">
								<div>
									<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Status</dt>
									<dd class="mt-1">
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
									<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Torque</dt>
									<dd class="mt-1 font-semibold">{event.result.torque.toFixed(2)} Nm</dd>
								</div>
								<div>
									<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Angle</dt>
									<dd class="mt-1 font-semibold">{event.result.angle.toFixed(1)}°</dd>
								</div>
								<div>
									<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Batch</dt>
									<dd class="mt-1 font-semibold">
										{event.result.batch_counter} / {event.result.batch_size}
									</dd>
								</div>
							</dl>
						{:else if event.type === 'MultiSpindleResultCompleted'}
							<dl class="mt-3 grid grid-cols-2 gap-3 text-sm md:grid-cols-4">
								<div>
									<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Status</dt>
									<dd class="mt-1">
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
									<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Spindles</dt>
									<dd class="mt-1 font-semibold">{event.result.spindle_count}</dd>
								</div>
								<div>
									<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Sync ID</dt>
									<dd class="mt-1 font-semibold">{event.result.sync_id}</dd>
								</div>
								<div>
									<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Result ID</dt>
									<dd class="mt-1 font-semibold">{event.result.result_id}</dd>
								</div>
							</dl>
						{:else if event.type === 'BatchCompleted'}
							<p class="mt-3 text-sm text-surface-600-300-token">Batch total: {event.total}</p>
						{:else if event.type === 'ToolStateChanged'}
							<p class="mt-3 text-sm text-surface-600-300-token">
								Tool {event.enabled ? 'enabled' : 'disabled'}
							</p>
						{:else if event.type === 'PsetChanged'}
							<p class="mt-3 text-sm text-surface-600-300-token">
								PSET switched to {event.pset_name} (ID {event.pset_id})
							</p>
						{:else if event.type === 'VehicleIdChanged'}
							<p class="mt-3 text-sm text-surface-600-300-token">VIN updated to {event.vin}</p>
						{:else if event.type === 'MultiSpindleStatusCompleted'}
							<p class="mt-3 text-sm text-surface-600-300-token">
								Status: {event.status.status === 0 ? 'Waiting' : event.status.status === 1 ? 'Running' : 'Completed'}
								· Sync ID {event.status.sync_id} · Spindles {event.status.spindle_count}
							</p>
						{/if}
					</article>
				{/each}
			</div>

			<div class="text-sm opacity-70">
				Showing {$filteredEvents.length} event(s)
			</div>
		{:else}
			<div class="card p-8 text-center">
				<p class="opacity-70">No events match your filters</p>
			</div>
		{/if}
	</section>
</div>
