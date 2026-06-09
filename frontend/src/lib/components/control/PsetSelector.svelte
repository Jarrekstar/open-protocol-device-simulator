<script lang="ts">
	import { deviceState } from '$lib/stores/device';
	import { api } from '$lib/api/client';
	import { showToast } from '$lib/stores/ui';
	import { Section, Badge } from '$lib/components/ui';
	import { getPsetTargets, formatErrorMessage } from '$lib/utils';
	import type { Pset } from '$lib/types';

	interface Props {
		psets: Pset[];
	}

	let { psets }: Props = $props();

	const currentPset = $derived(
		psets.find((p) => p.id === $deviceState?.current_pset_id)
	);

	const currentPsetTargets = $derived(
		currentPset ? getPsetTargets(currentPset) : null
	);
	const jobModeActive = $derived($deviceState?.current_job_status != null);

	async function handleSelectPset(psetId: number) {
		if (jobModeActive) return;
		try {
			await api.selectPset(psetId);
			showToast({ type: 'success', message: `PSET ${psetId} selected!` });
		} catch (error) {
			showToast({ type: 'error', message: formatErrorMessage('select PSET', error) });
		}
	}
</script>

<Section
	title="PSET Selection"
	description="Choose the active parameter set that governs the tightening strategy"
>
	<div class="grid gap-6 lg:grid-cols-2">
		<label class="label">
			<span>Select PSET</span>
			<select
				class="select"
				value={$deviceState?.current_pset_id || ''}
				onchange={(e) => handleSelectPset(Number(e.currentTarget.value))}
				disabled={jobModeActive}
			>
				{#each psets as pset}
					<option value={pset.id}>
						{pset.name} (ID {pset.id})
					</option>
				{/each}
			</select>
		</label>

		<div
			class="rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4"
		>
			{#if currentPsetTargets}
				<p class="text-xs uppercase tracking-wide text-surface-600-300-token mb-3">
					Configured Targets
				</p>
				<dl class="grid grid-cols-2 gap-4">
					<div>
						<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">
							Target Torque
						</dt>
						<dd class="mt-1 font-semibold">
							{currentPsetTargets.torque} Nm
						</dd>
					</div>
					<div>
						<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">
							Target Angle
						</dt>
						<dd class="mt-1 font-semibold">
							{currentPsetTargets.angle}°
						</dd>
					</div>
				</dl>
			{:else}
				<p class="text-sm opacity-70">No PSET selected. Choose one to view its targets.</p>
			{/if}
		</div>
	</div>
	{#if jobModeActive}
		<p class="mt-4 text-sm text-warning-600">
			PSET selection is controlled by JobMode. Restart or exit the active Job from the Jobs page.
		</p>
	{/if}
</Section>
