<script lang="ts">
	import { Modal } from '$lib/components/ui';
	import PsetForm from './PsetForm.svelte';
	import type { Pset } from '$lib/types';

	interface Props {
		open: boolean;
		mode: 'create' | 'edit';
		pset?: Pset | null;
		onsubmit: (data: Partial<Pset>) => Promise<void>;
		onclose: () => void;
	}

	let { open, mode, pset, onsubmit, onclose }: Props = $props();

	const title = $derived(mode === 'create' ? 'Create New PSET' : 'Edit PSET');
	const description = 'Define the torque and angle window that devices should target.';
</script>

<Modal {open} {title} {description} {onclose}>
	<PsetForm {mode} initialData={pset ?? undefined} {onsubmit} oncancel={onclose} />
</Modal>
