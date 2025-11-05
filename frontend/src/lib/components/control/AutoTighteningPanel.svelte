<script lang="ts">
	import { api } from '$lib/api/client';
	import { showToast } from '$lib/stores/ui';
	import { autoTighteningProgress } from '$lib/stores/tightening';
	import { Section, Button, FormField, Badge } from '$lib/components/ui';
	import { formatBatchCounter, formatErrorMessage } from '$lib/utils';

	let config = $state({
		interval_ms: 1000,
		duration_ms: 500,
		failure_rate: 0.0
	});
	let isLoading = $state(false);

	async function handleToggle() {
		isLoading = true;
		try {
			if ($autoTighteningProgress.running) {
				await api.stopAutoTightening();
				showToast({ type: 'success', message: 'Auto-tightening stopped!' });
			} else {
				await api.startAutoTightening(config);
				showToast({ type: 'success', message: 'Auto-tightening started!' });
			}
		} catch (error) {
			const action = $autoTighteningProgress.running ? 'stop' : 'start';
			showToast({ type: 'error', message: formatErrorMessage(`${action} auto-tightening`, error) });
		} finally {
			isLoading = false;
		}
	}
</script>

<Section
	title="Auto-Tightening"
	description="Schedule repeating cycles and monitor progress without leaving the control panel."
>
	<!-- Status Display -->
	<div class="rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4">
		<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
			<div>
				<p class="text-xs uppercase tracking-wide text-surface-600-300-token">Status</p>
				<div class="mt-2">
					<Badge
						variant={$autoTighteningProgress.running ? 'filled-success' : 'soft'}
					>
						{$autoTighteningProgress.running ? 'Running' : 'Stopped'}
					</Badge>
				</div>
			</div>
			{#if $autoTighteningProgress.target_size > 0 || $autoTighteningProgress.counter > 0}
				<div class="text-sm text-surface-600-300-token">
					<p class="font-semibold">Progress</p>
					<p class="text-surface-600-300-token">
						{formatBatchCounter(
							$autoTighteningProgress.counter,
							$autoTighteningProgress.target_size
						)}
					</p>
				</div>
			{/if}
		</div>
	</div>

	<!-- Configuration Form -->
	<form
		onsubmit={(e) => {
			e.preventDefault();
			handleToggle();
		}}
		class="space-y-6"
	>
		<div class="grid gap-4 md:grid-cols-3">
			<FormField
				label="Interval (ms)"
				type="number"
				bind:value={config.interval_ms}
				min={100}
				step={100}
			/>
			<FormField
				label="Duration (ms)"
				type="number"
				bind:value={config.duration_ms}
				min={100}
				step={100}
			/>
			<FormField
				label="Failure Rate (0.0 – 1.0)"
				type="number"
				bind:value={config.failure_rate}
				min={0}
				max={1}
				step={0.1}
			/>
		</div>

		<div class="flex justify-end">
			<Button
				type="submit"
				variant={$autoTighteningProgress.running ? 'filled-error' : 'filled-primary'}
				disabled={isLoading}
				class="sm:w-auto"
			>
				{#if isLoading}
					{$autoTighteningProgress.running ? 'Stopping...' : 'Starting...'}
				{:else}
					{$autoTighteningProgress.running ? 'Stop Auto-Tightening' : 'Start Auto-Tightening'}
				{/if}
			</Button>
		</div>
	</form>
</Section>
