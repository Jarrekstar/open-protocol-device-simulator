<script lang="ts">
	/**
	 * StatCard component for displaying metrics
	 */
	interface Props {
		/** Label for the stat */
		label: string;
		/** Value to display */
		value: string | number;
		/** Optional unit (e.g., "Nm", "°", "%") */
		unit?: string;
		/** Additional CSS classes */
		class?: string;
	}

	let { label, value, unit, class: className = '' }: Props = $props();
</script>

<div
	class="rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4 {className}"
>
	<p class="text-xs uppercase tracking-wide text-surface-600-300-token">{label}</p>
	<p class="mt-2 text-2xl font-semibold">
		{value}
		{#if unit}
			<span class="text-base font-normal">{unit}</span>
		{/if}
	</p>
</div>
