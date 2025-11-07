<script lang="ts">
	import { Button } from '$lib/components/ui';
	import { getPsetTargets } from '$lib/utils';
	import type { Pset } from '$lib/types';
	import RangeVisualizer from '$lib/components/ui/RangeVisualizer.svelte';

	interface Props {
		pset: Pset;
		onEdit: (pset: Pset) => void;
		onDelete: (id: number) => void;
		deleteConfirmId: number | null;
		onToggleDeleteConfirm: (id: number | null) => void;
	}

	let { pset, onEdit, onDelete, deleteConfirmId, onToggleDeleteConfirm }: Props = $props();

	const targets = $derived(getPsetTargets(pset));
	const showDeleteConfirm = $derived(deleteConfirmId === pset.id);
</script>

<article class="card flex h-full flex-col gap-5 p-6 hover-lift transition-smooth">
	<!-- Header -->
	<header class="space-y-2">
		<div class="flex items-start justify-between">
			<div class="flex-1">
				<h3 class="text-lg font-semibold text-surface-900 dark:text-surface-100">{pset.name}</h3>
				<p class="text-xs text-surface-500 dark:text-surface-400 mt-0.5">ID: {pset.id}</p>
			</div>
			<div
				class="flex-shrink-0 w-10 h-10 rounded-full bg-primary-100 dark:bg-primary-900 flex items-center justify-center"
			>
				<span class="text-lg">⚙️</span>
			</div>
		</div>
		{#if pset.description}
			<p class="text-sm text-surface-600 dark:text-surface-400">{pset.description}</p>
		{:else}
			<p class="text-sm text-surface-500 dark:text-surface-400 italic">No description</p>
		{/if}
	</header>

	<div class="divider-fade"></div>

	<!-- Visual Range Indicators -->
	<div class="space-y-4 flex-1">
		<RangeVisualizer
			min={pset.torque_min}
			max={pset.torque_max}
			target={targets?.torque ? parseFloat(targets.torque) : undefined}
			unit="Nm"
			label="Torque Range"
			color="rgb(var(--color-primary-500))"
		/>

		<RangeVisualizer
			min={pset.angle_min}
			max={pset.angle_max}
			target={targets?.angle ? parseFloat(targets.angle) : undefined}
			unit="°"
			label="Angle Range"
			color="rgb(var(--color-tertiary-500))"
		/>
	</div>

	<!-- Actions Footer -->
	<footer class="mt-auto space-y-3 pt-4 border-t border-surface-300 dark:border-surface-400">
		{#if !showDeleteConfirm}
			<div class="flex gap-2">
				<Button variant="filled-secondary" onclick={() => onEdit(pset)} class="flex-1">
					✏️ Edit
				</Button>
				<Button
					variant="ghost-surface"
					onclick={() => onToggleDeleteConfirm(pset.id)}
					class="flex-1 text-error-500 dark:text-error-400 hover:bg-error-500/10"
				>
					🗑️ Delete
				</Button>
			</div>
		{:else}
			<div class="space-y-3 rounded-lg bg-error-50 dark:bg-error-900/20 border border-error-500 p-4">
				<div class="flex items-start gap-2">
					<span class="text-error-500 text-lg">⚠️</span>
					<div class="flex-1">
						<p class="text-sm font-medium text-error-700 dark:text-error-400">
							Delete "{pset.name}"?
						</p>
						<p class="text-xs text-error-600 dark:text-error-500 mt-1">
							This action cannot be undone
						</p>
					</div>
				</div>
				<div class="flex gap-2">
					<Button variant="filled-error" onclick={() => onDelete(pset.id)} class="flex-1">
						Confirm Delete
					</Button>
					<Button
						variant="ghost-surface"
						onclick={() => onToggleDeleteConfirm(null)}
						class="flex-1"
					>
						Cancel
					</Button>
				</div>
			</div>
		{/if}
	</footer>
</article>
