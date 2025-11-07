<script lang="ts">
	import ChipFilter from './ChipFilter.svelte';

	interface Filter {
		id: string;
		label: string;
		count?: number;
		active: boolean;
	}

	interface Props {
		filters: Filter[];
		onFilterToggle: (filterId: string) => void;
		onClearAll?: () => void;
		className?: string;
	}

	let { filters, onFilterToggle, onClearAll, className = '' }: Props = $props();

	const activeFilters = $derived(filters.filter((f) => f.active));
	const hasActiveFilters = $derived(activeFilters.length > 0);
</script>

<div class="space-y-3 {className}">
	<div class="flex items-center justify-between">
		<h3 class="text-sm font-medium text-surface-700 dark:text-surface-300">Filters</h3>
		{#if hasActiveFilters && onClearAll}
			<button
				type="button"
				class="text-xs text-surface-600 dark:text-surface-400 hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
				onclick={onClearAll}
			>
				Clear all
			</button>
		{/if}
	</div>

	<div class="flex flex-wrap gap-2">
		{#each filters as filter}
			<ChipFilter
				label="{filter.label}{filter.count !== undefined ? ` (${filter.count})` : ''}"
				value={filter.id}
				active={filter.active}
				onClick={onFilterToggle}
				removable={false}
				variant={filter.active ? 'primary' : 'default'}
			/>
		{/each}
	</div>

	{#if hasActiveFilters}
		<div class="pt-2 border-t border-surface-300 dark:border-surface-400">
			<p class="text-xs text-surface-600 dark:text-surface-400">
				Active filters: {activeFilters.map((f) => f.label).join(', ')}
			</p>
		</div>
	{/if}
</div>
