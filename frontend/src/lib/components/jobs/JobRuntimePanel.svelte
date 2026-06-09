<script lang="ts">
	import { Badge, Button } from '$lib/components/ui';
	import { deviceState } from '$lib/stores/device';

	interface Props {
		onRestart: (id: number) => void;
		onClear: () => void;
	}

	let { onRestart, onClear }: Props = $props();
	const active = $derived($deviceState?.current_job_status != null);
	const running = $derived($deviceState?.current_job_status === 'running');
	const stepPercent = $derived(
		$deviceState && $deviceState.current_job_step_progress > 0
			? Math.min(
					100,
					($deviceState.current_job_step_progress /
						Math.max(1, $deviceState.current_job_step_batch_size)) *
						100
				)
			: 0
	);

	function restartActiveJob() {
		const jobId = $deviceState?.current_job_id;
		if (jobId != null) onRestart(jobId);
	}
	const totalPercent = $derived(
		$deviceState
			? Math.min(
					100,
					($deviceState.current_job_total_progress /
						Math.max(1, $deviceState.current_job_total_batch_size)) *
						100
				)
			: 0
	);
</script>

<section class="card p-6">
	<div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
		<div>
			<p class="text-xs uppercase tracking-wide opacity-60">JobMode</p>
			{#if active && $deviceState}
				<div class="mt-1 flex items-center gap-3">
					<h2 class="text-xl font-semibold">{$deviceState.current_job_name}</h2>
					<Badge
						variant={running
							? 'filled-primary'
							: $deviceState.current_job_status === 'ok'
								? 'filled-success'
								: 'filled-error'}
					>
						{$deviceState.current_job_status?.toUpperCase()}
					</Badge>
				</div>
				<p class="mt-1 text-sm opacity-70">
					Job {$deviceState.current_job_id?.toString().padStart(2, '0')} · PSET
					{$deviceState.current_pset_id}
				</p>
			{:else}
				<h2 class="mt-1 text-xl font-semibold">No configured Job selected</h2>
			{/if}
		</div>
		{#if active && $deviceState?.current_job_id != null}
			<div class="flex gap-2">
				<Button onclick={restartActiveJob}>Restart</Button>
				<Button variant="ghost-surface" onclick={onClear} disabled={running}>Exit JobMode</Button>
			</div>
		{/if}
	</div>

	{#if active && $deviceState}
		<div class="mt-5 grid gap-5 md:grid-cols-2">
			<div>
				<div class="mb-2 flex justify-between text-sm">
					<span>Step {$deviceState.current_job_step} / {$deviceState.current_job_total_steps}</span>
					<span>
						{$deviceState.current_job_step_progress} /
						{$deviceState.current_job_step_batch_size}
					</span>
				</div>
				<div class="h-2 overflow-hidden rounded-full bg-surface-200 dark:bg-surface-700">
					<div class="h-full bg-tertiary-500" style="width: {stepPercent}%"></div>
				</div>
			</div>
			<div>
				<div class="mb-2 flex justify-between text-sm">
					<span>Total progress</span>
					<span>
						{$deviceState.current_job_total_progress} /
						{$deviceState.current_job_total_batch_size}
					</span>
				</div>
				<div class="h-2 overflow-hidden rounded-full bg-surface-200 dark:bg-surface-700">
					<div class="h-full bg-primary-500" style="width: {totalPercent}%"></div>
				</div>
			</div>
		</div>
		{#if running}
			<p class="mt-4 text-sm text-warning-600">
				PSET selection, batch changes, resets, and manual torque/angle overrides are controlled by
				the running Job.
			</p>
		{/if}
	{/if}
</section>
