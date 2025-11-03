<script lang="ts">
	import { deviceState } from '$lib/stores/device';
	import { latestTighteningResult, tighteningStats } from '$lib/stores/tightening';
	import { events } from '$lib/stores/events';
	import { api } from '$lib/api/client';
	import { showToast } from '$lib/stores/ui';

	async function handleSimulateTightening() {
		try {
			await api.simulateTightening();
			showToast({ type: 'success', message: 'Tightening simulated!' });
		} catch (error) {
			showToast({ type: 'error', message: `Failed: ${error}` });
		}
	}
</script>

<svelte:head>
	<title>Dashboard - Device Simulator</title>
</svelte:head>

<div class="space-y-6">
	<h1 class="h1">Operations Overview</h1>

	<!-- Device + Latest Tightening Hero -->
	<section class="card p-6 space-y-6 lg:grid lg:grid-cols-2 lg:gap-8 lg:space-y-0">
		<div class="space-y-4">
			<div>
				<h2 class="h2">Device Status</h2>
				<p class="text-sm opacity-70">Live attributes from the connected simulator</p>
			</div>

			{#if $deviceState}
				<dl class="grid grid-cols-1 sm:grid-cols-2 gap-4">
					<div>
						<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Cell ID</dt>
						<dd class="mt-1 text-lg font-semibold">{$deviceState.cell_id}</dd>
					</div>
					<div>
						<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Channel ID</dt>
						<dd class="mt-1 text-lg font-semibold">{$deviceState.channel_id}</dd>
					</div>
					<div>
						<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Tool State</dt>
						<dd class="mt-1">
							<span class="badge" class:variant-filled-success={$deviceState.tool_enabled}>
								{$deviceState.tool_state}
							</span>
						</dd>
					</div>
					<div>
						<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Multi-Spindle</dt>
						<dd class="mt-1 flex items-center gap-2">
							<span class="badge" class:variant-filled-primary={$deviceState.multi_spindle_config.enabled}>
								{$deviceState.multi_spindle_config.enabled ? 'Enabled' : 'Disabled'}
							</span>
							{#if $deviceState.multi_spindle_config.enabled}
								<span class="text-sm opacity-70">
									{$deviceState.multi_spindle_config.spindle_count} spindles
								</span>
							{/if}
						</dd>
					</div>
				</dl>
			{:else}
				<p class="opacity-70">Loading device state...</p>
			{/if}
		</div>

		<div class="space-y-4">
			<div>
				<h2 class="h2">Latest Tightening</h2>
				<p class="text-sm opacity-70">Most recent cycle reported by the FSM</p>
			</div>

			{#if $latestTighteningResult}
				<div class="rounded-xl border border-surface-200-700-token bg-surface-100-800-token p-6">
					<div class="mb-4 flex items-center justify-between">
						<p class="text-sm uppercase tracking-wide text-surface-600-300-token">Status</p>
						<span
							class="badge"
							class:variant-filled-success={$latestTighteningResult.tightening_status}
							class:variant-filled-error={!$latestTighteningResult.tightening_status}
						>
							{$latestTighteningResult.tightening_status ? 'OK' : 'NOK'}
						</span>
					</div>
					<dl class="grid grid-cols-2 gap-4">
						<div>
							<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Torque</dt>
							<dd class="mt-1 text-2xl font-semibold">
								{$latestTighteningResult.torque.toFixed(2)} <span class="text-base font-normal">Nm</span>
							</dd>
						</div>
						<div>
							<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Angle</dt>
							<dd class="mt-1 text-2xl font-semibold">
								{$latestTighteningResult.angle.toFixed(1)}<span class="text-base font-normal">°</span>
							</dd>
						</div>
						<div>
							<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Batch Progress</dt>
							<dd class="mt-1 text-lg font-semibold">
								{$latestTighteningResult.batch_counter} / {$latestTighteningResult.batch_size}
							</dd>
						</div>
					</dl>
				</div>
			{:else}
				<div class="rounded-xl border border-dashed border-surface-200-700-token p-6 opacity-70">
					No tightening results yet
				</div>
			{/if}
		</div>
	</section>

	<!-- Quick Controls Toolbar -->
	<section class="card p-4 sm:p-6">
		<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
			<div>
				<h2 class="h3">Quick Controls</h2>
				<p class="text-sm opacity-70">Kick off a tightening cycle or open the full control panel</p>
			</div>
			<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:gap-3">
				<button class="btn variant-filled-primary w-full sm:w-auto" on:click={handleSimulateTightening}>
					Simulate Tightening
				</button>
				<a href="/control" class="btn variant-ghost-surface w-full sm:w-auto">
					More Controls →
				</a>
			</div>
		</div>
	</section>

	<!-- Metrics Strip -->
	<section class="card p-4 sm:p-6 space-y-4">
		<div class="flex items-center justify-between">
			<h2 class="h2">Performance Snapshot</h2>
			<span class="text-xs uppercase tracking-wide opacity-60">Rolling aggregates</span>
		</div>
		<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
			<div class="rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4">
				<p class="text-xs uppercase tracking-wide text-surface-600-300-token">Total Cycles</p>
				<p class="mt-2 text-2xl font-semibold">{$tighteningStats.total}</p>
			</div>
			<div class="rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4">
				<p class="text-xs uppercase tracking-wide text-surface-600-300-token">Success Rate</p>
				<p class="mt-2 text-2xl font-semibold">{$tighteningStats.successRate.toFixed(1)}%</p>
			</div>
			<div class="rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4">
				<p class="text-xs uppercase tracking-wide text-surface-600-300-token">Average Torque</p>
				<p class="mt-2 text-2xl font-semibold">
					{$tighteningStats.avgTorque.toFixed(2)} <span class="text-base font-normal">Nm</span>
				</p>
			</div>
			<div class="rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4">
				<p class="text-xs uppercase tracking-wide text-surface-600-300-token">Average Angle</p>
				<p class="mt-2 text-2xl font-semibold">
					{$tighteningStats.avgAngle.toFixed(1)}<span class="text-base font-normal">°</span>
				</p>
			</div>
		</div>
	</section>

	<!-- Recent Events Timeline -->
	<section class="card p-6">
		<div class="mb-6 flex items-center justify-between">
			<div>
				<h2 class="h2">Recent Events</h2>
				<p class="text-sm opacity-70">Quick view of the last five simulator events</p>
			</div>
			<a href="/events" class="btn variant-ghost-surface">View All Events →</a>
		</div>

		{#if $events.length > 0}
			<ol class="relative border-l border-surface-200-700-token pl-6">
				{#each $events.slice(0, 5) as event, index}
					<li class="mb-6 last:mb-0">
						<span
							class="absolute -left-3 mt-2 h-3 w-3 rounded-full border border-surface-100-800-token bg-primary-500"
							aria-hidden="true"
						/>
						<div class="flex items-center justify-between">
							<span class="badge variant-soft">{event.type}</span>
							<span class="text-xs uppercase tracking-wide text-surface-600-300-token">#{index + 1}</span>
						</div>
						<div class="mt-3 text-sm text-surface-600-300-token">
							{#if event.type === 'TighteningCompleted'}
								Torque {event.result.torque.toFixed(2)} Nm · Angle {event.result.angle.toFixed(1)}° ·
								Batch {event.result.batch_counter}/{event.result.batch_size}
							{:else if event.type === 'BatchCompleted'}
								Batch completed · Total {event.total}
							{:else if event.type === 'ToolStateChanged'}
								Tool {event.enabled ? 'enabled' : 'disabled'}
							{:else if event.type === 'PsetChanged'}
								PSET changed to {event.pset_name} (ID {event.pset_id})
							{:else if event.type === 'VehicleIdChanged'}
								VIN updated to {event.vin}
							{:else if event.type === 'MultiSpindleResultCompleted'}
								Multi-spindle result · {event.result.spindle_count} spindles · Sync {event.result.sync_id}
							{:else if event.type === 'MultiSpindleStatusCompleted'}
								Multi-spindle status · Sync {event.status.sync_id} · Spindles {event.status.spindle_count}
							{/if}
						</div>
					</li>
				{/each}
			</ol>
		{:else}
			<p class="opacity-70">No events yet</p>
		{/if}
	</section>
</div>
