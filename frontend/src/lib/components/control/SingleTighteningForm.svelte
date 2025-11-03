<script lang="ts">
	import { api } from '$lib/api/client';
	import { showToast } from '$lib/stores/ui';
	import { Section, Button, FormField } from '$lib/components/ui';
	import { getPsetTargets } from '$lib/utils';
	import type { Pset } from '$lib/types';

	interface Props {
		currentPset: Pset | undefined;
	}

	let { currentPset }: Props = $props();

	let usePsetValues = $state(true);
	let resultMode: 'auto' | 'ok' | 'nok' = $state('auto');
	let tighteningPayload = $state({
		torque: 12.5,
		angle: 40.0
	});

	const currentPsetTargets = $derived(
		currentPset ? getPsetTargets(currentPset) : null
	);

	async function handleSubmit() {
		try {
			let payload: any = {};

			if (!usePsetValues) {
				payload.torque = tighteningPayload.torque;
				payload.angle = tighteningPayload.angle;

				if (resultMode !== 'auto') {
					payload.ok = resultMode === 'ok';
				}
			}

			await api.simulateTightening(payload);
			showToast({ type: 'success', message: 'Tightening simulated!' });
		} catch (error) {
			showToast({ type: 'error', message: `Failed: ${error}` });
		}
	}
</script>

<Section
	title="Single Tightening"
	description="Run an ad-hoc cycle using the configured PSET or override values manually."
>
	<form
		onsubmit={(e) => {
			e.preventDefault();
			handleSubmit();
		}}
		class="space-y-6"
	>
		<!-- Toggle between PSET and Manual -->
		<div
			class="inline-flex rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-1"
		>
			<button
				type="button"
				class="px-4 py-2 text-sm font-semibold transition-colors rounded"
				class:bg-primary-500={usePsetValues}
				class:text-white={usePsetValues}
				class:opacity-60={!usePsetValues}
				onclick={() => (usePsetValues = true)}
			>
				Use PSET Values
			</button>
			<button
				type="button"
				class="px-4 py-2 text-sm font-semibold transition-colors rounded"
				class:bg-primary-500={!usePsetValues}
				class:text-white={!usePsetValues}
				class:opacity-60={usePsetValues}
				onclick={() => (usePsetValues = false)}
			>
				Manual Override
			</button>
		</div>

		{#if usePsetValues}
			<!-- PSET Mode -->
			<div
				class="rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4"
			>
				{#if currentPsetTargets}
					<p class="text-xs uppercase tracking-wide text-surface-600-300-token">
						PSET Target
					</p>
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
					<p class="mt-3 text-sm opacity-70">
						Outcome determined automatically by the FSM.
					</p>
				{:else}
					<p class="text-sm opacity-70">Select a PSET to preview its targets.</p>
				{/if}
			</div>
		{:else}
			<!-- Manual Override Mode -->
			<div class="grid gap-4 md:grid-cols-2">
				<FormField
					label="Torque (Nm)"
					type="number"
					bind:value={tighteningPayload.torque}
					step="0.1"
					min={0}
					max={100}
				/>
				<FormField
					label="Angle (degrees)"
					type="number"
					bind:value={tighteningPayload.angle}
					step="0.1"
					min={0}
					max={360}
				/>
			</div>

			<FormField
				label="Result Mode"
				type="select"
				bind:value={resultMode}
				options={[
					{ value: 'auto', label: 'Auto (FSM determines)' },
					{ value: 'ok', label: 'Force OK' },
					{ value: 'nok', label: 'Force NOK' }
				]}
			/>
		{/if}

		<Button type="submit" class="w-full sm:w-auto">
			Simulate Tightening
		</Button>
	</form>
</Section>
