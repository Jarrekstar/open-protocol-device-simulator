<script lang="ts">
	interface Props {
		min: number;
		max: number;
		target?: number;
		unit?: string;
		label?: string;
		color?: string;
		className?: string;
	}

	let {
		min,
		max,
		target,
		unit = '',
		label = '',
		color = 'rgb(var(--color-primary-500))',
		className = ''
	}: Props = $props();

	const range = $derived(max - min);
	const targetPosition = $derived(target !== undefined ? ((target - min) / range) * 100 : null);
</script>

<div class="space-y-2 {className}">
	{#if label}
		<div class="flex items-center justify-between text-xs">
			<span class="text-surface-600 dark:text-surface-400">{label}</span>
			{#if target !== undefined}
				<span class="font-semibold text-surface-900 dark:text-surface-100">
					Target: {target}{unit}
				</span>
			{/if}
		</div>
	{/if}

	<div class="relative h-8 rounded-lg bg-surface-200 dark:bg-surface-300 overflow-hidden">
		<!-- Range bar -->
		<div class="absolute inset-y-0 left-0 right-0 flex items-center px-2">
			<div class="w-full h-2 rounded-full" style="background: linear-gradient(90deg, {color}22, {color}66);"></div>
		</div>

		<!-- Target indicator -->
		{#if targetPosition !== null}
			<div
				class="absolute top-1/2 -translate-y-1/2 w-1 h-6 rounded-full shadow-md"
				style="left: {targetPosition}%; background: {color};"
			>
				<div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-3 h-3 rounded-full border-2 border-surface-50 dark:border-surface-900" style="background: {color};"></div>
			</div>
		{/if}

		<!-- Min/Max labels -->
		<div class="absolute inset-0 flex items-center justify-between px-2">
			<span class="text-2xs font-semibold text-surface-700 dark:text-surface-200">
				{min}{unit}
			</span>
			<span class="text-2xs font-semibold text-surface-700 dark:text-surface-200">
				{max}{unit}
			</span>
		</div>
	</div>
</div>
