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

	let failureConfig = {
		enabled: false,
		connection_health: 100,
		packet_loss_rate: 0.0,
		delay_min_ms: 0,
		delay_max_ms: 0,
		corruption_rate: 0.0,
		force_disconnect_rate: 0.0
	};

	let showAdvancedFailureSettings = false;

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

	async function handleUpdateFailureConfig() {
		try {
			const response = await api.updateFailureConfig({
				connection_health: failureConfig.connection_health
			});
			showToast({
				type: 'success',
				message: 'Failure injection configuration updated!'
			});

			// Update local config with confirmed state from backend
			// Convert rates from 0.0-1.0 (backend) to 0-100 (UI)
			failureConfig = {
				enabled: response.config.enabled,
				connection_health: response.config.connection_health,
				packet_loss_rate: response.config.packet_loss_rate * 100,
				delay_min_ms: response.config.delay_min_ms,
				delay_max_ms: response.config.delay_max_ms,
				corruption_rate: response.config.corruption_rate * 100,
				force_disconnect_rate: response.config.force_disconnect_rate * 100
			};

			// Refresh device state
			const state = await api.getDeviceState();
			deviceState.set(state);
		} catch (error) {
			showToast({ type: 'error', message: `Failed: ${error}` });
		}
	}

	async function handleUpdateAdvancedFailureConfig() {
		try {
			const response = await api.updateFailureConfig({
				enabled: failureConfig.enabled,
				packet_loss_rate: failureConfig.packet_loss_rate / 100.0,
				delay_min_ms: failureConfig.delay_min_ms,
				delay_max_ms: failureConfig.delay_max_ms,
				corruption_rate: failureConfig.corruption_rate / 100.0,
				force_disconnect_rate: failureConfig.force_disconnect_rate / 100.0
			});
			showToast({
				type: 'success',
				message: 'Advanced failure configuration updated!'
			});

			// Update local config with confirmed state from backend
			// Convert rates from 0.0-1.0 (backend) to 0-100 (UI)
			failureConfig = {
				enabled: response.config.enabled,
				connection_health: response.config.connection_health,
				packet_loss_rate: response.config.packet_loss_rate * 100,
				delay_min_ms: response.config.delay_min_ms,
				delay_max_ms: response.config.delay_max_ms,
				corruption_rate: response.config.corruption_rate * 100,
				force_disconnect_rate: response.config.force_disconnect_rate * 100
			};
		} catch (error) {
			showToast({ type: 'error', message: `Failed: ${error}` });
		}
	}

	async function handleResetFailureConfig() {
		failureConfig.connection_health = 100;
		await handleUpdateFailureConfig();
	}

	// Compute connection status message based on health
	$: connectionStatus = (() => {
		const health = failureConfig.connection_health;
		if (health === 100) return { label: 'Perfect', color: 'text-success-500' };
		if (health >= 75) return { label: 'Healthy', color: 'text-success-400' };
		if (health >= 50) return { label: 'Degraded', color: 'text-warning-500' };
		if (health >= 25) return { label: 'Unstable', color: 'text-warning-600' };
		return { label: 'Severe', color: 'text-error-500' };
	})();

	// Compute slider color based on health
	$: sliderColor = (() => {
		const health = failureConfig.connection_health;
		if (health >= 75) return 'accent-success-500';
		if (health >= 50) return 'accent-warning-500';
		if (health >= 25) return 'accent-warning-600';
		return 'accent-error-500';
	})();

	// Initialize multi-spindle config from device state once on mount
	import { onMount } from 'svelte';
	onMount(async () => {
		loadPsets();

		if ($deviceState) {
			multiSpindleConfig = {
				enabled: $deviceState.multi_spindle_config.enabled,
				spindle_count: $deviceState.multi_spindle_config.spindle_count,
				sync_id: $deviceState.multi_spindle_config.sync_id
			};

			// Load failure config and convert rates from 0.0-1.0 to 0-100 for UI
			failureConfig = {
				enabled: $deviceState.failure_config.enabled,
				connection_health: $deviceState.failure_config.connection_health,
				packet_loss_rate: $deviceState.failure_config.packet_loss_rate * 100,
				delay_min_ms: $deviceState.failure_config.delay_min_ms,
				delay_max_ms: $deviceState.failure_config.delay_max_ms,
				corruption_rate: $deviceState.failure_config.corruption_rate * 100,
				force_disconnect_rate: $deviceState.failure_config.force_disconnect_rate * 100
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

	<!-- Connection Failure Injection -->
	<div class="card p-4 lg:col-span-2">
		<h2 class="h2 mb-4">Connection Failure Injection</h2>

		<!-- Status Display -->
		<div class="mb-4 p-3 bg-surface-100-800-token rounded">
			<div class="flex justify-between items-center">
				<span class="font-semibold">Connection Status:</span>
				<span class="badge {connectionStatus.color}">
					{connectionStatus.label}
				</span>
			</div>
			{#if failureConfig.enabled && failureConfig.connection_health < 100}
				<div class="mt-2 text-sm text-warning-500">
					⚠️ Warning: Failure injection is active. TCP clients will experience connection issues.
				</div>
			{/if}
		</div>

		<!-- Connection Health Slider -->
		<div class="mb-6">
			<label class="label mb-2">
				<div class="flex justify-between items-center">
					<span class="font-semibold text-lg">Connection Health</span>
					<span class="text-2xl font-bold {connectionStatus.color}">
						{failureConfig.connection_health}%
					</span>
				</div>
			</label>
			<input
				type="range"
				class="w-full h-3 {sliderColor}"
				bind:value={failureConfig.connection_health}
				min="0"
				max="100"
				step="1"
				on:change={handleUpdateFailureConfig}
			/>
			<div class="flex justify-between text-xs text-surface-500 mt-1">
				<span>0% (Severe)</span>
				<span>25% (Unstable)</span>
				<span>50% (Degraded)</span>
				<span>75% (Healthy)</span>
				<span>100% (Perfect)</span>
			</div>
			<button
				type="button"
				class="btn variant-ghost-surface btn-sm mt-2"
				on:click={handleResetFailureConfig}
			>
				Reset to Perfect (100%)
			</button>
		</div>

		<!-- Connection Health Description -->
		<div class="mb-4 p-3 bg-surface-50-900-token rounded text-sm">
			<p class="font-semibold mb-2">Current Configuration:</p>
			<ul class="list-disc list-inside space-y-1">
				<li>Packet Loss: {(failureConfig.packet_loss_rate).toFixed(1)}%</li>
				<li>Delay Range: {failureConfig.delay_min_ms}-{failureConfig.delay_max_ms} ms</li>
				<li>Corruption Rate: {(failureConfig.corruption_rate).toFixed(1)}%</li>
				<li>Disconnect Rate: {(failureConfig.force_disconnect_rate).toFixed(1)}%</li>
			</ul>
		</div>

		<!-- Advanced Settings (Collapsible) -->
		<details class="mb-4" bind:open={showAdvancedFailureSettings}>
			<summary class="cursor-pointer font-semibold text-primary-500 hover:text-primary-600">
				Advanced Settings (Manual Override)
			</summary>
			<div class="mt-4 space-y-4 p-4 bg-surface-50-900-token rounded">
				<label class="flex items-center space-x-2">
					<input type="checkbox" class="checkbox" bind:checked={failureConfig.enabled} />
					<span class="font-semibold">Enable Failure Injection</span>
				</label>

				<label class="label">
					<span>Packet Loss Rate (%)</span>
					<input
						type="number"
						class="input"
						bind:value={failureConfig.packet_loss_rate}
						min="0"
						max="100"
						step="0.1"
					/>
				</label>

				<div class="grid grid-cols-2 gap-4">
					<label class="label">
						<span>Min Delay (ms)</span>
						<input
							type="number"
							class="input"
							bind:value={failureConfig.delay_min_ms}
							min="0"
							step="10"
						/>
					</label>
					<label class="label">
						<span>Max Delay (ms)</span>
						<input
							type="number"
							class="input"
							bind:value={failureConfig.delay_max_ms}
							min="0"
							step="10"
						/>
					</label>
				</div>

				<label class="label">
					<span>Corruption Rate (%)</span>
					<input
						type="number"
						class="input"
						bind:value={failureConfig.corruption_rate}
						min="0"
						max="100"
						step="0.1"
					/>
				</label>

				<label class="label">
					<span>Force Disconnect Rate (%)</span>
					<input
						type="number"
						class="input"
						bind:value={failureConfig.force_disconnect_rate}
						min="0"
						max="100"
						step="0.1"
					/>
				</label>

				<button
					type="button"
					class="btn variant-filled-primary w-full"
					on:click={handleUpdateAdvancedFailureConfig}
				>
					Apply Advanced Settings
				</button>
			</div>
		</details>
	</div>
</div>
