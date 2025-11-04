<script lang="ts">
	import { api } from '$lib/api/client';
	import { deviceState } from '$lib/stores/device';
	import { showToast } from '$lib/stores/ui';
	import { formatErrorMessage } from '$lib/utils';
	import type { Pset } from '$lib/types';
	import {
		PsetSelector,
		SingleTighteningForm,
		AutoTighteningPanel,
		MultiSpindleConfig,
		FailureInjectionPanel
	} from '$lib/components/control';
	import { onMount } from 'svelte';

	let psets: Pset[] = $state([]);

	const currentPset = $derived(
		psets.find((p) => p.id === $deviceState?.current_pset_id)
	);

	async function loadPsets() {
		try {
			psets = await api.getPsets();
		} catch (error) {
			console.error('Failed to load PSETs:', error);
			showToast({ type: 'error', message: formatErrorMessage('load PSETs', error) });
		}
	}

	onMount(() => {
		loadPsets();
	});
</script>

<svelte:head>
	<title>Control Panel - Device Simulator</title>
</svelte:head>

<h1 class="h1 mb-6">Control Panel</h1>

<div class="space-y-6">
	<PsetSelector {psets} />

	<SingleTighteningForm {currentPset} />

	<AutoTighteningPanel />

	<MultiSpindleConfig />

	<FailureInjectionPanel />
</div>
