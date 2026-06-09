<script lang="ts">
	import { Modal } from '$lib/components/ui';
	import type { Job, Pset } from '$lib/types';
	import JobForm from './JobForm.svelte';

	interface Props {
		open: boolean;
		mode: 'create' | 'edit';
		job?: Job | null;
		psets: Pset[];
		channelId: number;
		onsubmit: (job: Job) => Promise<void>;
		onclose: () => void;
	}

	let { open, mode, job, psets, channelId, onsubmit, onclose }: Props = $props();
</script>

<Modal
	{open}
	title={mode === 'create' ? 'Create Job' : `Edit Job ${job?.id.toString().padStart(2, '0')}`}
	description="Configure the revision 1 Job definition and ordered PSET steps."
	onclose={onclose}
	maxWidth="xl"
>
	{#key `${mode}-${job?.id ?? 'new'}-${open}`}
		<JobForm
			{mode}
			initialData={job ?? undefined}
			{psets}
			{channelId}
			{onsubmit}
			oncancel={onclose}
		/>
	{/key}
</Modal>
