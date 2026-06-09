<script lang="ts">
	import { Badge, Button } from '$lib/components/ui';
	import type { Job } from '$lib/types';

	interface Props {
		job: Job;
		active: boolean;
		running: boolean;
		onEdit: (job: Job) => void;
		onDelete: (id: number) => void;
		onSelect: (id: number) => void;
	}

	let { job, active, running, onEdit, onDelete, onSelect }: Props = $props();
	const total = $derived(job.steps.reduce((sum, step) => sum + step.batch_size, 0));
</script>

<article class="card flex h-full flex-col gap-4 p-5">
	<header class="flex items-start justify-between gap-3">
		<div>
			<div class="flex items-center gap-2">
				<h3 class="text-lg font-semibold">{job.name}</h3>
				{#if active}<Badge variant={running ? 'filled-primary' : 'soft'}>Active</Badge>{/if}
			</div>
			<p class="text-xs opacity-60">Job {job.id.toString().padStart(2, '0')}</p>
		</div>
		<Badge variant="soft">{job.steps.length} steps</Badge>
	</header>

	<dl class="grid grid-cols-2 gap-3 text-sm">
		<div>
			<dt class="opacity-60">Total tightenings</dt>
			<dd class="font-semibold">{total}</dd>
		</div>
		<div>
			<dt class="opacity-60">Count mode</dt>
			<dd class="font-semibold">{job.batch_count_mode === 0 ? 'OK only' : 'OK + NOK'}</dd>
		</div>
		<div>
			<dt class="opacity-60">Completion</dt>
			<dd class="font-semibold">
				{job.repeat_job ? 'Repeat' : job.lock_at_job_done ? 'Lock tool' : 'Stop'}
			</dd>
		</div>
		<div>
			<dt class="opacity-60">PSET sequence</dt>
			<dd class="font-semibold">{job.steps.map((step) => step.pset_id).join(' → ')}</dd>
		</div>
	</dl>

	<footer class="mt-auto flex flex-wrap gap-2 border-t border-surface-200-700-token pt-4">
		<Button onclick={() => onSelect(job.id)} disabled={running} class="flex-1">
			Select
		</Button>
		<Button
			variant="filled-secondary"
			onclick={() => onEdit(job)}
			disabled={active && running}
		>
			Edit
		</Button>
		<Button
			variant="ghost-surface"
			onclick={() => onDelete(job.id)}
			disabled={active && running}
			class="text-error-500"
		>
			Delete
		</Button>
	</footer>
</article>
