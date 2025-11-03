<script lang="ts">
	import { Card, Button } from '$lib/components/ui';
	import { getPsetTargets } from '$lib/utils';
	import type { Pset } from '$lib/types';

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

<article class="card flex h-full flex-col gap-4 p-6">
	<header class="rounded-lg bg-surface-100-800-token p-4">
		<h3 class="text-lg font-semibold">{pset.name}</h3>
		<p class="text-sm opacity-60">ID: {pset.id}</p>
	</header>

	{#if pset.description}
		<p class="text-sm text-surface-600-300-token">{pset.description}</p>
	{:else}
		<p class="text-sm opacity-70">No description provided.</p>
	{/if}

	<dl
		class="grid grid-cols-2 gap-4 rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4 text-sm"
	>
		<div>
			<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">
				Torque Range
			</dt>
			<dd class="mt-1 font-semibold text-surface-600-300-token">
				{pset.torque_min} – {pset.torque_max} Nm
			</dd>
		</div>
		<div>
			<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Angle Range</dt>
			<dd class="mt-1 font-semibold text-surface-600-300-token">
				{pset.angle_min}° – {pset.angle_max}°
			</dd>
		</div>
		<div>
			<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">
				Target Torque
			</dt>
			<dd class="mt-1 font-semibold text-surface-600-300-token">
				{targets?.torque} Nm
			</dd>
		</div>
		<div>
			<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">
				Target Angle
			</dt>
			<dd class="mt-1 font-semibold text-surface-600-300-token">
				{targets?.angle}°
			</dd>
		</div>
	</dl>

	<footer class="mt-auto space-y-3 border-t border-surface-200-700-token pt-4">
		<div class="flex gap-2">
			<Button variant="filled-secondary" onclick={() => onEdit(pset)} class="btn-sm flex-1">
				Edit
			</Button>
			<Button
				variant="ghost-surface"
				onclick={() => onToggleDeleteConfirm(showDeleteConfirm ? null : pset.id)}
				class="btn-sm flex-1 text-error-500 hover:bg-error-500/10"
			>
				Delete
			</Button>
		</div>

		{#if showDeleteConfirm}
			<div
				class="space-y-3 rounded-lg border border-error-500 bg-surface-100-800-token p-3 text-sm text-error-600"
			>
				<p>Delete {pset.name}? This action cannot be undone.</p>
				<div class="flex gap-2">
					<Button
						variant="filled-error"
						onclick={() => onDelete(pset.id)}
						class="btn-sm flex-1"
					>
						Confirm
					</Button>
					<Button
						variant="ghost-surface"
						onclick={() => onToggleDeleteConfirm(null)}
						class="btn-sm flex-1"
					>
						Cancel
					</Button>
				</div>
			</div>
		{/if}
	</footer>
</article>
