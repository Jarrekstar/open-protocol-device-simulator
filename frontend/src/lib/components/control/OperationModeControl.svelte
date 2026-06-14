<script lang="ts">
	import { api } from '$lib/api/client';
	import { deviceState } from '$lib/stores/device';
	import { showToast } from '$lib/stores/ui';
	import { refreshDeviceState } from '$lib/stores/websocket';
	import { formatErrorMessage } from '$lib/utils';
	import { Badge, Button, Section } from '$lib/components/ui';
	import type { OperationMode, OperationModeRequest } from '$lib/types';

	let selectedMode: OperationMode = $state('pset');
	let isApplying = $state(false);
	let observedMode: OperationMode | null = null;

	const modes: { id: OperationMode; label: string; description: string }[] = [
		{ id: 'pset', label: 'PSET', description: 'PSET-controlled tightening' },
		{ id: 'batch', label: 'Batch', description: 'MID 0019 batch tracking' },
		{ id: 'job', label: 'Job', description: 'Job-controlled sequence' }
	];

	const changingRunningJob = $derived(
		$deviceState?.operation_mode === 'job' &&
			$deviceState.current_job_status === 'running' &&
			String(selectedMode) !== 'job'
	);

	$effect(() => {
		const state = $deviceState;
		if (!state) return;
		if (state.operation_mode !== observedMode) {
			observedMode = state.operation_mode;
			selectedMode = state.operation_mode;
		}
	});

	async function applyMode() {
		const request: OperationModeRequest = { mode: selectedMode };

		isApplying = true;
		try {
			const response = await api.setOperationMode(request);
			await refreshDeviceState();
			showToast({ type: 'success', message: response.message });
		} catch (error) {
			showToast({ type: 'error', message: formatErrorMessage('change operation mode', error) });
		} finally {
			isApplying = false;
		}
	}
</script>

<Section
	title="Operation Mode"
	description="Choose controller behavior and the mode-specific command set."
>
	<div class="grid gap-3 md:grid-cols-3">
		{#each modes as mode}
			<button
				type="button"
				class="rounded-lg border p-4 text-left transition-colors"
				class:border-primary-500={selectedMode === mode.id}
				class:bg-primary-500={selectedMode === mode.id}
				class:text-white={selectedMode === mode.id}
				class:border-surface-200-700-token={selectedMode !== mode.id}
				class:bg-surface-100-800-token={selectedMode !== mode.id}
				onclick={() => (selectedMode = mode.id)}
				aria-pressed={selectedMode === mode.id}
			>
				<span class="block font-semibold">{mode.label}</span>
				<span class="mt-1 block text-sm opacity-75">{mode.description}</span>
			</button>
		{/each}
	</div>

	{#if selectedMode === 'batch'}
		<p class="text-sm opacity-70">
			Batch mode tracks tightening progress against the batch size supplied via MID 0019;
			until then, tightenings are reported without batch tracking. Job selection (MID 0038)
			is rejected with error 20.
		</p>
	{:else if selectedMode === 'job'}
		<p class="text-sm opacity-70">
			Job mode accepts Job selection (MID 0038) and Job batch increment (MID 0128). Direct
			PSET selection (MID 0018) is rejected with error 03 and batch configuration (MID
			0019-0020) with error 01 while Jobs govern the device.
		</p>
	{:else}
		<p class="text-sm opacity-70">
			PSET mode runs single tightenings. Setting a batch size via MID 0019 switches to Batch
			mode, per the spec. Job selection (MID 0038) is rejected with error 20. Uploads and
			subscriptions (including Job info, MID 0030-0037) are accepted in every mode.
		</p>
	{/if}

	{#if changingRunningJob}
		<p class="rounded-md bg-warning-50 p-3 text-sm text-warning-700 dark:bg-warning-900/20">
			Applying this change abandons the currently running Job.
		</p>
	{/if}

	<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
		<div class="flex items-center gap-2 text-sm">
			<span class="opacity-70">Current:</span>
			<Badge variant="soft">{$deviceState?.operation_mode?.toUpperCase() ?? 'LOADING'}</Badge>
			{#if $deviceState?.operation_mode === 'batch'}
				{#if $deviceState.batch_size > 0}
					<span class="opacity-70">
						{$deviceState.batch_counter} / {$deviceState.batch_size}
					</span>
				{:else}
					<span class="opacity-70">No batch configured</span>
				{/if}
			{/if}
		</div>
		<Button onclick={applyMode} disabled={isApplying}>
			{isApplying ? 'Applying...' : 'Apply Mode'}
		</Button>
	</div>
</Section>
