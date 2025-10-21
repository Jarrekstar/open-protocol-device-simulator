<script lang="ts">
	import { api } from '$lib/api/client';
	import { showToast } from '$lib/stores/ui';
	import { deviceState } from '$lib/stores/device';
	import { autoTighteningProgress } from '$lib/stores/tightening';
	import type { Pset } from '$lib/types';

	let autoTighteningConfig = {
		interval_ms: 1000,
		duration_ms: 500,
		failure_rate: 0.0
	};

	let multiSpindleConfig = {
		enabled: false,
		spindle_count: 4,
		sync_id: 1
	};

	let usePsetValues = true; // Default to using PSET
	let resultMode: 'auto' | 'ok' | 'nok' = 'auto'; // 3-way result choice

	let tighteningPayload = {
		torque: 12.5,
		angle: 40.0
	};

	let psets: Pset[] = [];

	// Compute current PSET target values for display
	$: currentPsetTargets = $deviceState?.current_pset_id
		? (() => {
				const pset = psets.find((p) => p.id === $deviceState?.current_pset_id);
				if (pset) {
					return {
						torque: ((pset.torque_min + pset.torque_max) / 2).toFixed(1),
						angle: ((pset.angle_min + pset.angle_max) / 2).toFixed(1)
					};
				}
				return null;
		  })()
		: null;

	async function loadPsets() {
		try {
			psets = await api.getPsets();
		} catch (error) {
			console.error('Failed to load PSETs:', error);
		}
	}

	async function handleSelectPset(psetId: number) {
		try {
			await api.selectPset(psetId);
			showToast({ type: 'success', message: `PSET ${psetId} selected!` });
		} catch (error) {
			showToast({ type: 'error', message: `Failed to select PSET: ${error}` });
		}
	}

	async function handleSimulateTightening() {
		try {
			let payload: any = {};

			if (!usePsetValues) {
				// Manual override mode - send torque and angle
				payload.torque = tighteningPayload.torque;
				payload.angle = tighteningPayload.angle;

				// Add OK/NOK override if not auto
				if (resultMode !== 'auto') {
					payload.ok = resultMode === 'ok';
				}
			}
			// If usePsetValues is true, send empty object (use PSET)

			await api.simulateTightening(payload);
			showToast({ type: 'success', message: 'Tightening simulated!' });
		} catch (error) {
			showToast({ type: 'error', message: `Failed: ${error}` });
		}
	}

	async function handleStartAutoTightening() {
		try {
			await api.startAutoTightening(autoTighteningConfig);
			showToast({ type: 'success', message: 'Auto-tightening started!' });
		} catch (error) {
			showToast({ type: 'error', message: `Failed: ${error}` });
		}
	}

	async function handleStopAutoTightening() {
		try {
			await api.stopAutoTightening();
			showToast({ type: 'success', message: 'Auto-tightening stopped!' });
		} catch (error) {
			showToast({ type: 'error', message: `Failed: ${error}` });
		}
	}

	async function handleConfigureMultiSpindle() {
		try {
			await api.configureMultiSpindle(multiSpindleConfig);
			showToast({
				type: 'success',
				message: `Multi-spindle ${multiSpindleConfig.enabled ? 'enabled' : 'disabled'}!`
			});

			// Refresh device state to ensure UI reflects backend state
			const state = await api.getDeviceState();
			deviceState.set(state);

			// Update form with confirmed state from backend
			multiSpindleConfig = {
				enabled: state.multi_spindle_config.enabled,
				spindle_count: state.multi_spindle_config.spindle_count,
				sync_id: state.multi_spindle_config.sync_id
			};
		} catch (error) {
			showToast({ type: 'error', message: `Failed: ${error}` });
		}
	}

	// Initialize multi-spindle config from device state once on mount
	import { onMount } from 'svelte';
	onMount(() => {
		loadPsets();

		if ($deviceState) {
			multiSpindleConfig = {
				enabled: $deviceState.multi_spindle_config.enabled,
				spindle_count: $deviceState.multi_spindle_config.spindle_count,
				sync_id: $deviceState.multi_spindle_config.sync_id
			};
		}
	});
</script>

<svelte:head>
	<title>Control Panel - Device Simulator</title>
</svelte:head>

<h1 class="h1 mb-6">Control Panel</h1>

<!-- PSET Selector -->
<div class="card p-4 mb-4">
	<h2 class="h2 mb-4">Parameter Set (PSET)</h2>
	<div class="flex gap-4 items-end">
		<label class="label flex-1">
			<span class="mb-2 block">Current PSET</span>
			<select
				class="select"
				value={$deviceState?.current_pset_id ?? ''}
				on:change={(e) => {
					const value = e.currentTarget.value;
					if (value) {
						handleSelectPset(Number(value));
					}
				}}
			>
				<option value="">-- Select PSET --</option>
				{#each psets as pset}
					<option value={pset.id}>
						{pset.name} (Torque: {pset.torque_min}-{pset.torque_max} Nm, Angle: {pset.angle_min}°-{pset.angle_max}°)
					</option>
				{/each}
			</select>
		</label>
		<a href="/psets" class="btn variant-ghost-surface">
			Manage PSETs
		</a>
	</div>
	{#if $deviceState?.current_pset_id}
		<div class="mt-3 text-sm text-surface-600-300-token">
			Active PSET: <strong>#{$deviceState.current_pset_id}</strong>
			{#if $deviceState.current_pset_name}
				- {$deviceState.current_pset_name}
			{/if}
		</div>
	{/if}
</div>

<div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
	<!-- Single Tightening Simulation -->
	<div class="card p-4">
		<h2 class="h2 mb-4">Simulate Tightening</h2>
		<form on:submit|preventDefault={handleSimulateTightening} class="space-y-4">
			<!-- Use PSET checkbox -->
			<label class="flex items-center space-x-2">
				<input type="checkbox" class="checkbox" bind:checked={usePsetValues} />
				<span class="font-semibold">Use PSET values</span>
			</label>

			{#if usePsetValues}
				<!-- Show PSET values (read-only) -->
				<div class="p-3 bg-surface-100-800-token rounded">
					{#if currentPsetTargets}
						<p class="text-sm">
							<span class="font-semibold">PSET Target:</span>
						</p>
						<p class="text-sm mt-1">Torque: {currentPsetTargets.torque} Nm</p>
						<p class="text-sm">Angle: {currentPsetTargets.angle}°</p>
						<p class="text-sm mt-2 text-surface-500">Result determined by FSM</p>
					{:else}
						<p class="text-sm text-surface-500">No PSET selected</p>
					{/if}
				</div>
			{:else}
				<!-- Manual override fields -->
				<label class="label">
					<span>Torque (Nm)</span>
					<input
						type="number"
						class="input"
						bind:value={tighteningPayload.torque}
						step="0.1"
						min="0"
						max="100"
					/>
				</label>
				<label class="label">
					<span>Angle (degrees)</span>
					<input
						type="number"
						class="input"
						bind:value={tighteningPayload.angle}
						step="0.1"
						min="0"
						max="360"
					/>
				</label>

				<!-- Result Mode Selector -->
				<label class="label">
					<span>Result Mode</span>
					<select class="select" bind:value={resultMode}>
						<option value="auto">Auto (FSM determines)</option>
						<option value="ok">Force OK</option>
						<option value="nok">Force NOK</option>
					</select>
				</label>
			{/if}

			<button type="submit" class="btn variant-filled-primary w-full">
				Simulate Tightening
			</button>
		</form>
	</div>

	<!-- Auto-Tightening Controls -->
	<div class="card p-4">
		<h2 class="h2 mb-4">Auto-Tightening</h2>

		<div class="mb-4 p-3 bg-surface-100-800-token rounded">
			<div class="flex justify-between">
				<span class="font-semibold">Status:</span>
				<span
					class="badge"
					class:variant-filled-success={$autoTighteningProgress.running}
					class:variant-filled-surface={!$autoTighteningProgress.running}
				>
					{$autoTighteningProgress.running ? 'Running' : 'Stopped'}
				</span>
			</div>
			{#if $autoTighteningProgress.target_size > 0 || $autoTighteningProgress.counter > 0}
				<div class="flex justify-between mt-2">
					<span class="font-semibold">Progress:</span>
					<span>
						{$autoTighteningProgress.counter} / {$autoTighteningProgress.target_size}
					</span>
				</div>
			{/if}
		</div>

		<form on:submit|preventDefault={handleStartAutoTightening} class="space-y-4">
			<label class="label">
				<span>Interval (ms)</span>
				<input
					type="number"
					class="input"
					bind:value={autoTighteningConfig.interval_ms}
					min="100"
					step="100"
				/>
			</label>
			<label class="label">
				<span>Duration (ms)</span>
				<input
					type="number"
					class="input"
					bind:value={autoTighteningConfig.duration_ms}
					min="100"
					step="100"
				/>
			</label>
			<label class="label">
				<span>Failure Rate (0.0 - 1.0)</span>
				<input
					type="number"
					class="input"
					bind:value={autoTighteningConfig.failure_rate}
					min="0"
					max="1"
					step="0.1"
				/>
			</label>

			<div class="flex gap-2">
				<button type="submit" class="btn variant-filled-primary flex-1">Start</button>
				<button
					type="button"
					class="btn variant-filled-error flex-1"
					on:click={handleStopAutoTightening}
				>
					Stop
				</button>
			</div>
		</form>
	</div>

	<!-- Multi-Spindle Configuration -->
	<div class="card p-4">
		<h2 class="h2 mb-4">Multi-Spindle Configuration</h2>
		<form on:submit|preventDefault={handleConfigureMultiSpindle} class="space-y-4">
			<label class="flex items-center space-x-2">
				<input type="checkbox" class="checkbox" bind:checked={multiSpindleConfig.enabled} />
				<span>Enable Multi-Spindle Mode</span>
			</label>

			{#if multiSpindleConfig.enabled}
				<label class="label">
					<span>Spindle Count (2-16)</span>
					<input
						type="number"
						class="input"
						bind:value={multiSpindleConfig.spindle_count}
						min="2"
						max="16"
					/>
				</label>
				<label class="label">
					<span>Sync ID</span>
					<input type="number" class="input" bind:value={multiSpindleConfig.sync_id} min="1" />
				</label>
			{/if}

			<button type="submit" class="btn variant-filled-primary w-full">
				Apply Configuration
			</button>
		</form>
	</div>
</div>
