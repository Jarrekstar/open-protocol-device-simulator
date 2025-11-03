<script lang="ts">
	import { api } from '$lib/api/client';
	import { showToast } from '$lib/stores/ui';
	import { deviceState } from '$lib/stores/device';
	import { Section, Button, FormField } from '$lib/components/ui';
	import { onMount } from 'svelte';

	let config = $state({
		enabled: false,
		spindle_count: 4,
		sync_id: 1
	});

	async function handleSubmit() {
		try {
			await api.configureMultiSpindle(config);
			showToast({
				type: 'success',
				message: `Multi-spindle ${config.enabled ? 'enabled' : 'disabled'}!`
			});

			// Refresh device state to ensure UI reflects backend state
			const state = await api.getDeviceState();
			deviceState.set(state);

			// Update form with confirmed state from backend
			config = {
				enabled: state.multi_spindle_config.enabled,
				spindle_count: state.multi_spindle_config.spindle_count,
				sync_id: state.multi_spindle_config.sync_id
			};
		} catch (error) {
			showToast({ type: 'error', message: `Failed: ${error}` });
		}
	}

	// Initialize from device state on mount
	onMount(() => {
		if ($deviceState) {
			config = {
				enabled: $deviceState.multi_spindle_config.enabled,
				spindle_count: $deviceState.multi_spindle_config.spindle_count,
				sync_id: $deviceState.multi_spindle_config.sync_id
			};
		}
	});
</script>

<Section
	title="Multi-Spindle Configuration"
	description="Enable synchronized spindles and adjust counts before orchestrating multi-tool cycles."
>
	<form
		onsubmit={(e) => {
			e.preventDefault();
			handleSubmit();
		}}
		class="space-y-4"
	>
		<label
			class="flex items-center gap-3 rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-3"
		>
			<input type="checkbox" class="checkbox" bind:checked={config.enabled} />
			<span class="block">
				<p class="font-semibold">Enable Multi-Spindle Mode</p>
				<p class="text-sm opacity-70">
					When active, simulated spindles fire together per sync ID.
				</p>
			</span>
		</label>

		{#if config.enabled}
			<div class="grid gap-4 md:grid-cols-2">
				<FormField
					label="Spindle Count (2-16)"
					type="number"
					bind:value={config.spindle_count}
					min={2}
					max={16}
				/>
				<FormField
					label="Sync ID"
					type="number"
					bind:value={config.sync_id}
					min={1}
				/>
			</div>
		{/if}

		<div class="flex justify-end">
			<Button type="submit" class="sm:w-auto">
				Apply Configuration
			</Button>
		</div>
	</form>
</Section>
