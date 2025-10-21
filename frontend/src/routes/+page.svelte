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

<div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
	<!-- Device Status Card -->
	<div class="card p-4">
		<h2 class="h2 mb-4">Device Status</h2>
		{#if $deviceState}
			<dl class="space-y-2">
				<div class="flex justify-between">
					<dt class="font-semibold">Cell ID:</dt>
					<dd>{$deviceState.cell_id}</dd>
				</div>
				<div class="flex justify-between">
					<dt class="font-semibold">Channel ID:</dt>
					<dd>{$deviceState.channel_id}</dd>
				</div>
				<div class="flex justify-between">
					<dt class="font-semibold">Tool State:</dt>
					<dd>
						<span class="badge" class:variant-filled-success={$deviceState.tool_enabled}>
							{$deviceState.tool_state}
						</span>
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="font-semibold">Multi-Spindle:</dt>
					<dd>
						<span
							class="badge"
							class:variant-filled-primary={$deviceState.multi_spindle_config.enabled}
						>
							{$deviceState.multi_spindle_config.enabled ? 'Enabled' : 'Disabled'}
						</span>
						{#if $deviceState.multi_spindle_config.enabled}
							<span class="text-sm ml-2">
								({$deviceState.multi_spindle_config.spindle_count} spindles)
							</span>
						{/if}
					</dd>
				</div>
			</dl>
		{:else}
			<p class="text-surface-500">Loading device state...</p>
		{/if}
	</div>

	<!-- Latest Tightening Result Card -->
	<div class="card p-4">
		<h2 class="h2 mb-4">Latest Tightening</h2>
		{#if $latestTighteningResult}
			<dl class="space-y-2">
				<div class="flex justify-between">
					<dt class="font-semibold">Status:</dt>
					<dd>
						<span
							class="badge"
							class:variant-filled-success={$latestTighteningResult.tightening_status}
							class:variant-filled-error={!$latestTighteningResult.tightening_status}
						>
							{$latestTighteningResult.tightening_status ? 'OK' : 'NOK'}
						</span>
					</dd>
				</div>
				<div class="flex justify-between">
					<dt class="font-semibold">Torque:</dt>
					<dd>{$latestTighteningResult.torque.toFixed(2)} Nm</dd>
				</div>
				<div class="flex justify-between">
					<dt class="font-semibold">Angle:</dt>
					<dd>{$latestTighteningResult.angle.toFixed(1)}°</dd>
				</div>
				<div class="flex justify-between">
					<dt class="font-semibold">Batch:</dt>
					<dd>{$latestTighteningResult.batch_counter} / {$latestTighteningResult.batch_size}</dd>
				</div>
			</dl>
		{:else}
			<p class="text-surface-500">No tightening results yet</p>
		{/if}
	</div>

	<!-- Quick Controls Card -->
	<div class="card p-4">
		<h2 class="h2 mb-4">Quick Controls</h2>
		<div class="flex flex-col gap-2">
			<button class="btn variant-filled-primary" on:click={handleSimulateTightening}>
				Simulate Tightening
			</button>
			<a href="/control" class="btn variant-ghost-surface"> More Controls → </a>
		</div>
	</div>

	<!-- Statistics Card -->
	<div class="card p-4">
		<h2 class="h2 mb-4">Statistics</h2>
		<dl class="space-y-2">
			<div class="flex justify-between">
				<dt class="font-semibold">Total:</dt>
				<dd>{$tighteningStats.total}</dd>
			</div>
			<div class="flex justify-between">
				<dt class="font-semibold">Success Rate:</dt>
				<dd>{$tighteningStats.successRate.toFixed(1)}%</dd>
			</div>
			<div class="flex justify-between">
				<dt class="font-semibold">Avg Torque:</dt>
				<dd>{$tighteningStats.avgTorque.toFixed(2)} Nm</dd>
			</div>
			<div class="flex justify-between">
				<dt class="font-semibold">Avg Angle:</dt>
				<dd>{$tighteningStats.avgAngle.toFixed(1)}°</dd>
			</div>
		</dl>
	</div>

	<!-- Recent Events Card -->
	<div class="card p-4 lg:col-span-2">
		<h2 class="h2 mb-4">Recent Events</h2>
		{#if $events.length > 0}
			<div class="space-y-2">
				{#each $events.slice(0, 5) as event}
					<div class="flex items-center justify-between border-b border-surface-300-600-token pb-2">
						<span class="badge variant-soft">{event.type}</span>
						<span class="text-sm text-surface-500">
							{#if event.type === 'TighteningCompleted'}
								Torque: {event.result.torque.toFixed(2)} Nm
							{:else if event.type === 'BatchCompleted'}
								Total: {event.total}
							{/if}
						</span>
					</div>
				{/each}
			</div>
			<a href="/events" class="btn variant-ghost-surface w-full mt-4"> View All Events → </a>
		{:else}
			<p class="text-surface-500">No events yet</p>
		{/if}
	</div>
</div>
