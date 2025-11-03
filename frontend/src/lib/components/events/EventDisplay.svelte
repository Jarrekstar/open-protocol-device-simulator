<script lang="ts">
	import type { SimulatorEvent } from '$lib/types';
	import TighteningEvent from './TighteningEvent.svelte';
	import BatchEvent from './BatchEvent.svelte';
	import ToolStateEvent from './ToolStateEvent.svelte';
	import PsetChangedEvent from './PsetChangedEvent.svelte';
	import VehicleIdEvent from './VehicleIdEvent.svelte';
	import MultiSpindleResultEvent from './MultiSpindleResultEvent.svelte';
	import MultiSpindleStatusEvent from './MultiSpindleStatusEvent.svelte';
	import AutoTighteningProgressEvent from './AutoTighteningProgressEvent.svelte';

	interface Props {
		event: SimulatorEvent;
	}

	let { event }: Props = $props();
</script>

{#if event.type === 'TighteningCompleted'}
	<TighteningEvent result={event.result} />
{:else if event.type === 'BatchCompleted'}
	<BatchEvent total={event.total} />
{:else if event.type === 'ToolStateChanged'}
	<ToolStateEvent enabled={event.enabled} />
{:else if event.type === 'PsetChanged'}
	<PsetChangedEvent psetId={event.pset_id} psetName={event.pset_name} />
{:else if event.type === 'VehicleIdChanged'}
	<VehicleIdEvent vin={event.vin} />
{:else if event.type === 'MultiSpindleResultCompleted'}
	<MultiSpindleResultEvent result={event.result} />
{:else if event.type === 'MultiSpindleStatusCompleted'}
	<MultiSpindleStatusEvent status={event.status} />
{:else if event.type === 'AutoTighteningProgress'}
	<AutoTighteningProgressEvent
		counter={event.counter}
		targetSize={event.target_size}
		running={event.running}
	/>
{/if}
