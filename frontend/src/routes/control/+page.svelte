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

<div class="space-y-6">
	<!-- PSET -->
	<section class="card p-6 space-y-6">
		<header class="flex flex-col gap-2 md:flex-row md:items-end md:justify-between">
			<div>
				<h2 class="h2">Parameter Set (PSET)</h2>
				<p class="text-sm opacity-70">Select the active torque/angle window for upcoming cycles</p>
			</div>
			<a href="/psets" class="btn variant-ghost-surface md:w-auto">Manage PSETs</a>
		</header>

		<div class="grid gap-6 md:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
			<label class="label">
				<span class="mb-2 block text-surface-600-300-token">Current PSET</span>
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

			<div class="rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4">
				{#if $deviceState?.current_pset_id && currentPsetTargets}
					<p class="text-xs uppercase tracking-wide text-surface-600-300-token">Active PSET</p>
					<p class="mt-2 text-lg font-semibold">
						#{$deviceState.current_pset_id}
						{#if $deviceState.current_pset_name}
							· {$deviceState.current_pset_name}
						{/if}
					</p>
					<dl class="mt-3 grid grid-cols-2 gap-4 text-sm">
						<div>
							<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Target Torque</dt>
							<dd class="mt-1 font-semibold">
								{currentPsetTargets.torque} Nm
							</dd>
						</div>
						<div>
							<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Target Angle</dt>
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
	</section>

	<!-- Single Tightening -->
	<section class="card p-6 space-y-6">
		<header>
			<h2 class="h2">Single Tightening</h2>
			<p class="text-sm opacity-70">
				Run an ad-hoc cycle using the configured PSET or override values manually.
			</p>
		</header>

		<form on:submit|preventDefault={handleSimulateTightening} class="space-y-6">
			<div class="inline-flex rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-1">
				<button
					type="button"
					class="px-4 py-2 text-sm font-semibold transition-colors rounded"
					class:bg-primary-500={usePsetValues}
					class:text-white={usePsetValues}
					class:opacity-60={!usePsetValues}
					on:click={() => (usePsetValues = true)}
				>
					Use PSET Values
				</button>
				<button
					type="button"
					class="px-4 py-2 text-sm font-semibold transition-colors rounded"
					class:bg-primary-500={!usePsetValues}
					class:text-white={!usePsetValues}
					class:opacity-60={usePsetValues}
					on:click={() => (usePsetValues = false)}
				>
					Manual Override
				</button>
			</div>

			{#if usePsetValues}
				<div class="rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4">
					{#if currentPsetTargets}
						<p class="text-xs uppercase tracking-wide text-surface-600-300-token">PSET Target</p>
						<div class="mt-2 grid grid-cols-1 gap-3 sm:grid-cols-2">
							<div>
								<p class="text-sm opacity-70">Torque</p>
								<p class="text-xl font-semibold">
									{currentPsetTargets.torque} Nm
								</p>
							</div>
							<div>
								<p class="text-sm opacity-70">Angle</p>
								<p class="text-xl font-semibold">
									{currentPsetTargets.angle}°
								</p>
							</div>
						</div>
						<p class="mt-3 text-sm opacity-70">Outcome determined automatically by the FSM.</p>
					{:else}
						<p class="text-sm opacity-70">Select a PSET to preview its targets.</p>
					{/if}
				</div>
			{:else}
				<div class="grid gap-4 md:grid-cols-2">
					<label class="label">
						<span class="text-surface-600-300-token">Torque (Nm)</span>
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
						<span class="text-surface-600-300-token">Angle (degrees)</span>
						<input
							type="number"
							class="input"
							bind:value={tighteningPayload.angle}
							step="0.1"
							min="0"
							max="360"
						/>
					</label>
				</div>

				<label class="label">
					<span class="text-surface-600-300-token">Result Mode</span>
					<select class="select" bind:value={resultMode}>
						<option value="auto">Auto (FSM determines)</option>
						<option value="ok">Force OK</option>
						<option value="nok">Force NOK</option>
					</select>
				</label>
			{/if}

			<button type="submit" class="btn variant-filled-primary w-full sm:w-auto">
				Simulate Tightening
			</button>
		</form>
	</section>

	<!-- Auto-Tightening -->
	<section class="card p-6 space-y-6">
		<header>
			<h2 class="h2">Auto-Tightening</h2>
			<p class="text-sm opacity-70">
				Schedule repeating cycles and monitor progress without leaving the control panel.
			</p>
		</header>

		<div class="rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4">
			<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
				<div>
					<p class="text-xs uppercase tracking-wide text-surface-600-300-token">Status</p>
					<span
						class="badge mt-2"
						class:variant-filled-success={$autoTighteningProgress.running}
						class:variant-filled-surface={!$autoTighteningProgress.running}
					>
						{$autoTighteningProgress.running ? 'Running' : 'Stopped'}
					</span>
				</div>
				{#if $autoTighteningProgress.target_size > 0 || $autoTighteningProgress.counter > 0}
					<div class="text-sm text-surface-600-300-token">
						<p class="font-semibold">Progress</p>
						<p class="text-surface-600-300-token">
							{$autoTighteningProgress.counter} / {$autoTighteningProgress.target_size}
						</p>
					</div>
				{/if}
			</div>
		</div>

		<form on:submit|preventDefault={handleStartAutoTightening} class="space-y-6">
			<div class="grid gap-4 md:grid-cols-3">
				<label class="label">
					<span class="text-surface-600-300-token">Interval (ms)</span>
					<input
						type="number"
						class="input"
						bind:value={autoTighteningConfig.interval_ms}
						min="100"
						step="100"
					/>
				</label>
				<label class="label">
					<span class="text-surface-600-300-token">Duration (ms)</span>
					<input
						type="number"
						class="input"
						bind:value={autoTighteningConfig.duration_ms}
						min="100"
						step="100"
					/>
				</label>
				<label class="label">
					<span class="text-surface-600-300-token">Failure Rate (0.0 – 1.0)</span>
					<input
						type="number"
						class="input"
						bind:value={autoTighteningConfig.failure_rate}
						min="0"
						max="1"
						step="0.1"
					/>
				</label>
			</div>

			<div class="flex flex-col gap-2 sm:flex-row sm:justify-end">
				<button type="submit" class="btn variant-filled-primary sm:w-auto">Start</button>
				<button type="button" class="btn variant-filled-error sm:w-auto" on:click={handleStopAutoTightening}>
					Stop
				</button>
			</div>
		</form>
	</section>

	<!-- Multi-Spindle -->
	<section class="card p-6 space-y-6">
		<header>
			<h2 class="h2">Multi-Spindle Configuration</h2>
			<p class="text-sm opacity-70">
				Enable synchronized spindles and adjust counts before orchestrating multi-tool cycles.
			</p>
		</header>

		<form on:submit|preventDefault={handleConfigureMultiSpindle} class="space-y-4">
			<label class="flex items-center gap-3 rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-3">
				<input type="checkbox" class="checkbox" bind:checked={multiSpindleConfig.enabled} />
				<span class="block">
					<p class="font-semibold">Enable Multi-Spindle Mode</p>
					<p class="text-sm opacity-70">When active, simulated spindles fire together per sync ID.</p>
				</span>
			</label>

			{#if multiSpindleConfig.enabled}
				<div class="grid gap-4 md:grid-cols-2">
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
				</div>
			{/if}

			<div class="flex justify-end">
				<button type="submit" class="btn variant-filled-primary sm:w-auto">
					Apply Configuration
				</button>
			</div>
		</form>
	</section>

	<!-- Failure Injection -->
	<section class="card p-6 space-y-6">
		<header>
			<h2 class="h2">Connection Failure Injection</h2>
			<p class="text-sm opacity-70">
				Dial down connection health to test client resilience. Advanced overrides are available if needed.
			</p>
		</header>

		<div class="rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4">
			<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
				<div>
					<p class="text-xs uppercase tracking-wide text-surface-600-300-token">Connection Status</p>
					<p class="mt-2 text-xl font-semibold {connectionStatus.color}">{connectionStatus.label}</p>
				</div>
				{#if failureConfig.enabled && failureConfig.connection_health < 100}
					<div class="flex items-start gap-2 rounded-md bg-surface-100-800-token p-3 text-sm text-warning-600">
						<span aria-hidden="true">⚠️</span>
						<span>Failure injection is active. TCP clients may experience disruptions.</span>
					</div>
				{/if}
			</div>
		</div>

		<div class="space-y-3">
			<label class="flex items-center justify-between text-sm font-semibold text-surface-600-300-token">
				<span>Connection Health</span>
				<span class="{connectionStatus.color} text-lg">
					{failureConfig.connection_health}%
				</span>
			</label>
			<div class="flex items-center gap-3">
				<span class="text-xs uppercase tracking-wide text-surface-600-300-token w-10 text-left">0%</span>
				<input
					type="range"
					class="flex-1 {sliderColor}"
					bind:value={failureConfig.connection_health}
					min="0"
					max="100"
					step="1"
					on:change={handleUpdateFailureConfig}
				/>
				<span class="text-xs uppercase tracking-wide text-surface-600-300-token w-10 text-right">100%</span>
			</div>
			<div class="grid grid-cols-5 text-xs text-surface-600-300-token">
				<span>Severe</span>
				<span class="text-center">Unstable</span>
				<span class="text-center">Degraded</span>
				<span class="text-center">Healthy</span>
				<span class="text-right">Perfect</span>
			</div>
			<div class="flex justify-end">
				<button type="button" class="btn variant-ghost-surface btn-sm" on:click={handleResetFailureConfig}>
					Reset to 100%
				</button>
			</div>
		</div>

		<div class="rounded-lg border border-dashed border-surface-200-700-token bg-surface-100-800-token p-4 text-sm">
			<p class="font-semibold">Current Configuration</p>
			<dl class="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-2">
				<div>
					<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Packet Loss</dt>
					<dd class="mt-1 font-semibold">
						{(failureConfig.packet_loss_rate).toFixed(1)}%
					</dd>
				</div>
				<div>
					<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Delay Range</dt>
					<dd class="mt-1 font-semibold">
						{failureConfig.delay_min_ms}-{failureConfig.delay_max_ms} ms
					</dd>
				</div>
				<div>
					<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Corruption Rate</dt>
					<dd class="mt-1 font-semibold">
						{(failureConfig.corruption_rate).toFixed(1)}%
					</dd>
				</div>
				<div>
					<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Disconnect Rate</dt>
					<dd class="mt-1 font-semibold">
						{(failureConfig.force_disconnect_rate).toFixed(1)}%
					</dd>
				</div>
			</dl>
		</div>

		<details class="rounded-lg border border-surface-200-700-token p-4" bind:open={showAdvancedFailureSettings}>
			<summary class="cursor-pointer text-sm font-semibold text-primary-500 hover:text-primary-600">
				Advanced Settings (Manual Override)
			</summary>
			<div class="mt-4 space-y-4">
				<label class="flex items-center gap-3 rounded-lg bg-surface-100-800-token p-3">
					<input type="checkbox" class="checkbox" bind:checked={failureConfig.enabled} />
					<span class="font-semibold">Enable Failure Injection</span>
				</label>

				<label class="label">
					<span class="text-surface-600-300-token">Packet Loss Rate (%)</span>
					<input
						type="number"
						class="input"
						bind:value={failureConfig.packet_loss_rate}
						min="0"
						max="100"
						step="0.1"
					/>
				</label>

				<div class="grid gap-4 md:grid-cols-2">
					<label class="label">
						<span class="text-surface-600-300-token">Min Delay (ms)</span>
						<input
							type="number"
							class="input"
							bind:value={failureConfig.delay_min_ms}
							min="0"
							step="10"
						/>
					</label>
					<label class="label">
						<span class="text-surface-600-300-token">Max Delay (ms)</span>
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
					<span class="text-surface-600-300-token">Corruption Rate (%)</span>
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
					<span class="text-surface-600-300-token">Force Disconnect Rate (%)</span>
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
					class="btn variant-filled-primary w-full sm:w-auto"
					on:click={handleUpdateAdvancedFailureConfig}
				>
					Apply Advanced Settings
				</button>
			</div>
		</details>
	</section>
</div>
