<script lang="ts">
	import { formatBatchCounter } from '$lib/utils';

	interface Props {
		counter: number;
		targetSize: number;
		running: boolean;
	}

	let { counter, targetSize, running }: Props = $props();
</script>

<div class="text-sm text-surface-600-300-token">
	Auto-tightening {running ? 'in progress' : 'stopped'} · {formatBatchCounter(
		counter,
		targetSize
	)}
</div>
