<script lang="ts">
	import Sparkline from './Sparkline.svelte';

	interface Props {
		label: string;
		value: string | number;
		unit?: string;
		trend?: 'up' | 'down' | 'neutral';
		trendData?: number[];
		color?: string;
		icon?: string;
		className?: string;
	}

	let {
		label,
		value,
		unit = '',
		trend = 'neutral',
		trendData = [],
		color = 'rgb(var(--color-primary-500))',
		icon,
		className = ''
	}: Props = $props();

	const trendIcon = $derived.by(() => {
		if (trend === 'up') return '↑';
		if (trend === 'down') return '↓';
		return '−';
	});

	const trendClass = $derived.by(() => {
		if (trend === 'up') return 'trend-up';
		if (trend === 'down') return 'trend-down';
		return 'trend-neutral';
	});
</script>

<div class="flex flex-col {className}">
	<div class="flex items-start justify-between mb-2">
		<div class="flex-1">
			<div class="text-sm text-surface-600 dark:text-surface-400 mb-1">
				{label}
			</div>
			<div class="flex items-baseline gap-2">
				<div class="text-2xl font-semibold text-surface-900 dark:text-surface-100">
					{value}{#if unit}<span class="text-sm text-surface-500 ml-1">{unit}</span>{/if}
				</div>
				{#if trend !== 'neutral'}
					<span class="text-sm font-medium {trendClass}">
						{trendIcon}
					</span>
				{/if}
			</div>
		</div>
		{#if icon}
			<div class="text-2xl opacity-50">
				{icon}
			</div>
		{/if}
	</div>

	{#if trendData.length > 0}
		<div class="mt-2">
			<Sparkline
				data={trendData}
				width={120}
				height={32}
				{color}
				filled={true}
				showDot={true}
			/>
		</div>
	{/if}
</div>
