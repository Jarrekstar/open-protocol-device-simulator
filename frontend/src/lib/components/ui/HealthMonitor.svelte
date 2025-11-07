<script lang="ts">
	import StatusIndicator from './StatusIndicator.svelte';

	interface Props {
		connectionHealth: number; // 0-100
		packetLoss?: number;
		latency?: number;
		className?: string;
	}

	let { connectionHealth = 100, packetLoss = 0, latency = 0, className = '' }: Props = $props();

	const healthStatus = $derived.by(() => {
		if (connectionHealth >= 80) return 'success';
		if (connectionHealth >= 50) return 'warning';
		return 'error';
	});

	const healthLabel = $derived.by(() => {
		if (connectionHealth >= 80) return 'Excellent';
		if (connectionHealth >= 50) return 'Degraded';
		return 'Poor';
	});
</script>

<div class="space-y-3 {className}">
	<div class="flex items-center justify-between">
		<span class="text-sm font-medium text-surface-700 dark:text-surface-300">Connection Health</span>
		<StatusIndicator
			status={healthStatus}
			label={healthLabel}
			size="sm"
		/>
	</div>

	<!-- Health bar -->
	<div class="relative h-2 bg-surface-200 dark:bg-surface-300 rounded-full overflow-hidden">
		<div
			class="absolute top-0 left-0 h-full transition-all duration-500 rounded-full {healthStatus ===
			'success'
				? 'bg-success-500'
				: healthStatus === 'warning'
					? 'bg-warning-500'
					: 'bg-error-500'}"
			style="width: {connectionHealth}%"
		></div>
	</div>

	<!-- Metrics grid -->
	<div class="grid grid-cols-3 gap-3 text-center">
		<div class="space-y-1">
			<div class="text-2xl font-semibold text-surface-900 dark:text-surface-100">
				{connectionHealth}<span class="text-sm text-surface-500">%</span>
			</div>
			<div class="text-2xs text-surface-500 dark:text-surface-400">Quality</div>
		</div>

		{#if packetLoss !== undefined}
			<div class="space-y-1">
				<div class="text-2xl font-semibold text-surface-900 dark:text-surface-100">
					{packetLoss.toFixed(1)}<span class="text-sm text-surface-500">%</span>
				</div>
				<div class="text-2xs text-surface-500 dark:text-surface-400">Loss</div>
			</div>
		{/if}

		{#if latency !== undefined}
			<div class="space-y-1">
				<div class="text-2xl font-semibold text-surface-900 dark:text-surface-100">
					{latency}<span class="text-sm text-surface-500">ms</span>
				</div>
				<div class="text-2xs text-surface-500 dark:text-surface-400">Latency</div>
			</div>
		{/if}
	</div>
</div>
