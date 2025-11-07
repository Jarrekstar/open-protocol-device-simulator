<script lang="ts">
	import { deviceState } from '$lib/stores/device';
	import {
		latestTighteningResult,
		tighteningStats,
		torqueSparkline,
		angleSparkline,
		successRateSparkline
	} from '$lib/stores/tightening';
	import { events } from '$lib/stores/events';
	import { connectionHealth, latency, packetLoss } from '$lib/stores/websocket';
	import { api } from '$lib/api/client';
	import { showToast } from '$lib/stores/ui';
	import { Badge } from '$lib/components/ui';
	import { EventTimeline } from '$lib/components/events';
	import { formatPercentage, formatErrorMessage } from '$lib/utils';
	import DataCard from '$lib/components/ui/DataCard.svelte';
	import MetricSparkline from '$lib/components/ui/MetricSparkline.svelte';
	import HealthMonitor from '$lib/components/ui/HealthMonitor.svelte';
	import StatusIndicator from '$lib/components/ui/StatusIndicator.svelte';

	let isSimulating = $state(false);

	async function handleSimulateTightening() {
		isSimulating = true;
		try {
			await api.simulateTightening();
			showToast({ type: 'success', message: 'Tightening simulated!' });
		} catch (error) {
			showToast({ type: 'error', message: formatErrorMessage('simulate tightening', error) });
		} finally {
			isSimulating = false;
		}
	}
</script>

<svelte:head>
	<title>Dashboard - Device Simulator</title>
</svelte:head>

<div class="space-y-8 animate-fade-in">
	<!-- Header with Quick Action -->
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-3xl font-semibold text-surface-900 dark:text-surface-100">
				Mission Control
			</h1>
			<p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
				Real-time operations overview and system status
			</p>
		</div>
		<button
			class="btn variant-filled-primary {isSimulating ? 'loading' : ''}"
			onclick={handleSimulateTightening}
			disabled={isSimulating}
		>
			{isSimulating ? 'Simulating...' : '⚡ Simulate Tightening'}
		</button>
	</div>

	<!-- Hero Section - 3-column layout -->
	<div class="grid grid-cols-1 lg:grid-cols-3 gap-6 stagger-children">
		<!-- Device Status -->
		<DataCard title="Device Status" subtitle="Live configuration" hover={true}>
			{#if $deviceState}
				<div class="space-y-4">
					<dl class="space-y-3">
						<div class="flex items-center justify-between">
							<dt class="text-xs uppercase tracking-wide text-surface-600 dark:text-surface-400">
								Cell ID
							</dt>
							<dd class="font-semibold text-surface-900 dark:text-surface-100">
								{$deviceState.cell_id}
							</dd>
						</div>
						<div class="divider-fade"></div>
						<div class="flex items-center justify-between">
							<dt class="text-xs uppercase tracking-wide text-surface-600 dark:text-surface-400">
								Channel ID
							</dt>
							<dd class="font-semibold text-surface-900 dark:text-surface-100">
								{$deviceState.channel_id}
							</dd>
						</div>
						<div class="divider-fade"></div>
						<div class="flex items-center justify-between">
							<dt class="text-xs uppercase tracking-wide text-surface-600 dark:text-surface-400">
								Tool State
							</dt>
							<dd>
								<StatusIndicator
									status={$deviceState.tool_enabled ? 'success' : 'warning'}
									label={$deviceState.tool_state}
									size="sm"
									showLabel={true}
								/>
							</dd>
						</div>
						<div class="divider-fade"></div>
						<div class="flex items-center justify-between">
							<dt class="text-xs uppercase tracking-wide text-surface-600 dark:text-surface-400">
								Active PSET
							</dt>
							<dd class="flex items-center gap-2">
								{#if $deviceState.current_pset_name}
									<Badge variant="filled-primary">
										{$deviceState.current_pset_name}
									</Badge>
									<span class="text-xs text-surface-500">
										(ID {$deviceState.current_pset_id})
									</span>
								{:else}
									<Badge variant="soft">None</Badge>
								{/if}
							</dd>
						</div>
						<div class="divider-fade"></div>
						<div class="flex items-center justify-between">
							<dt class="text-xs uppercase tracking-wide text-surface-600 dark:text-surface-400">
								Multi-Spindle
							</dt>
							<dd class="flex items-center gap-2">
								<Badge
									variant={$deviceState.multi_spindle_config.enabled ? 'filled-primary' : 'soft'}
								>
									{$deviceState.multi_spindle_config.enabled ? 'Active' : 'Inactive'}
								</Badge>
								{#if $deviceState.multi_spindle_config.enabled}
									<span class="text-xs text-surface-500">
										({$deviceState.multi_spindle_config.spindle_count})
									</span>
								{/if}
							</dd>
						</div>
					</dl>
				</div>
			{:else}
				<div class="flex items-center justify-center py-8 opacity-70">
					<div class="loading-skeleton" style="width: 100%; height: 120px;"></div>
				</div>
			{/if}
		</DataCard>

		<!-- Latest Tightening -->
		<DataCard title="Latest Tightening" subtitle="Most recent cycle" hover={true} accent={true}>
			{#if $latestTighteningResult}
				<div class="space-y-4">
					<div class="flex items-center justify-between">
						<span class="text-sm text-surface-600 dark:text-surface-400">Result</span>
						<StatusIndicator
							status={$latestTighteningResult.tightening_status ? 'success' : 'error'}
							label={$latestTighteningResult.tightening_status ? 'OK' : 'NOK'}
							size="md"
						/>
					</div>
					<div class="grid grid-cols-2 gap-4 pt-2">
						<div>
							<div class="text-3xl font-semibold text-surface-900 dark:text-surface-100">
								{$latestTighteningResult.torque.toFixed(2)}
							</div>
							<div class="text-xs text-surface-500 dark:text-surface-400 mt-1">
								Torque (Nm)
							</div>
						</div>
						<div>
							<div class="text-3xl font-semibold text-surface-900 dark:text-surface-100">
								{$latestTighteningResult.angle.toFixed(1)}
							</div>
							<div class="text-xs text-surface-500 dark:text-surface-400 mt-1">
								Angle (°)
							</div>
						</div>
					</div>
					<div class="divider-fade"></div>
					<div class="space-y-2">
						<div class="flex items-center justify-between text-sm">
							<span class="text-surface-600 dark:text-surface-400">Batch Progress</span>
							<span class="font-semibold text-surface-900 dark:text-surface-100">
								{$latestTighteningResult.batch_counter} / {$latestTighteningResult.batch_size}
							</span>
						</div>
						<div class="w-full bg-surface-200 dark:bg-surface-700 rounded-full h-2 overflow-hidden">
							<div
								class="bg-primary-500 h-full transition-all duration-300 ease-out"
								style="width: {($latestTighteningResult.batch_counter / $latestTighteningResult.batch_size) * 100}%"
							></div>
						</div>
						<div class="text-xs text-center text-surface-500 dark:text-surface-400">
							{Math.round(($latestTighteningResult.batch_counter / $latestTighteningResult.batch_size) * 100)}% Complete
						</div>
					</div>
				</div>
			{:else}
				<div class="rounded-lg border-2 border-dashed border-surface-300 dark:border-surface-400 p-8 text-center">
					<p class="text-sm text-surface-500 dark:text-surface-400">
						No tightening results yet
					</p>
				</div>
			{/if}
		</DataCard>

		<!-- Connection Health Monitor -->
		<DataCard title="Connection Health" subtitle="Network quality metrics" hover={true}>
			<HealthMonitor
				connectionHealth={$connectionHealth}
				packetLoss={$packetLoss}
				latency={$latency}
			/>
		</DataCard>
	</div>

	<!-- Performance Metrics with Sparklines -->
	<DataCard padding="lg">
		{#snippet headerAction()}
			<div class="flex items-center gap-2">
				<span class="text-2xs uppercase tracking-wide text-surface-500 dark:text-surface-400">
					Live Metrics
				</span>
				<div class="h-2 w-2 rounded-full bg-success-500 animate-pulse"></div>
			</div>
		{/snippet}

		<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
			<div class="metric-card card p-5 space-y-3">
				<div class="flex items-center justify-between">
					<span class="text-sm text-surface-600 dark:text-surface-400">Total Cycles</span>
					<span class="text-2xl">🔄</span>
				</div>
				<div class="text-3xl font-semibold text-surface-900 dark:text-surface-100">
					{$tighteningStats.total}
				</div>
			</div>

			<div class="metric-card card p-5">
				<MetricSparkline
					label="Success Rate"
					value={formatPercentage($tighteningStats.successRate)}
					trend={$tighteningStats.successRate >= 90 ? 'up' : $tighteningStats.successRate < 70 ? 'down' : 'neutral'}
					trendData={$successRateSparkline}
					color="rgb(var(--color-success-500))"
					icon="✓"
				/>
			</div>

			<div class="metric-card card p-5">
				<MetricSparkline
					label="Avg. Torque"
					value={$tighteningStats.avgTorque.toFixed(2)}
					unit="Nm"
					trend="neutral"
					trendData={$torqueSparkline}
					color="rgb(var(--color-primary-500))"
					icon="⚙️"
				/>
			</div>

			<div class="metric-card card p-5">
				<MetricSparkline
					label="Avg. Angle"
					value={$tighteningStats.avgAngle.toFixed(1)}
					unit="°"
					trend="neutral"
					trendData={$angleSparkline}
					color="rgb(var(--color-tertiary-500))"
					icon="↻"
				/>
			</div>
		</div>
	</DataCard>

	<!-- Recent Events Timeline -->
	<DataCard title="Recent Activity" subtitle="Last 5 events from the simulator">
		{#snippet headerAction()}
			<a href="/events" class="btn variant-ghost-surface btn-sm">
				View All →
			</a>
		{/snippet}

		{#if $events.length > 0}
			<EventTimeline events={$events} limit={5} showNumbers={true} />
		{:else}
			<div class="py-12">
				<div class="text-center text-surface-500 dark:text-surface-400">
					<p class="text-sm">No events recorded yet</p>
					<p class="text-xs mt-1">Simulate a tightening to see events appear here</p>
				</div>
			</div>
		{/if}
	</DataCard>

	<!-- Quick Navigation Footer -->
	<div class="glass rounded-xl p-6">
		<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
			<div>
				<h3 class="text-lg font-medium text-surface-900 dark:text-surface-100">
					Need More Control?
				</h3>
				<p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
					Access advanced features and configurations
				</p>
			</div>
			<div class="flex flex-wrap gap-3">
				<a href="/control" class="btn variant-filled-primary">
					Control Panel
				</a>
				<a href="/psets" class="btn variant-ghost-surface">
					PSETs
				</a>
				<a href="/events" class="btn variant-ghost-surface">
					Event Log
				</a>
			</div>
		</div>
	</div>
</div>
