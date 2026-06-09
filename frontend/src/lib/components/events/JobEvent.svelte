<script lang="ts">
	import { Badge } from '$lib/components/ui';
	import type { JobRuntimeState } from '$lib/types';

	interface Props {
		action: string;
		state: JobRuntimeState;
		previousStep?: number;
	}

	let { action, state, previousStep }: Props = $props();
</script>

<div class="space-y-2 text-sm">
	<div class="flex flex-wrap items-center gap-2">
		<strong>{action}: {state.job_name}</strong>
		<Badge
			variant={state.status === 'running'
				? 'filled-primary'
				: state.status === 'ok'
					? 'filled-success'
					: 'filled-error'}
		>
			{state.status.toUpperCase()}
		</Badge>
	</div>
	<p class="opacity-70">
		Job {state.job_id.toString().padStart(2, '0')} ·
		{#if previousStep != null}step {previousStep} to {/if}step {state.current_step}/{state.total_steps}
		· PSET {state.current_pset_id} · {state.total_progress}/{state.total_batch_size}
	</p>
</div>
