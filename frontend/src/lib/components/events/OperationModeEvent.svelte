<script lang="ts">
	import type { OperationMode } from '$lib/types';

	interface Props {
		mode: OperationMode;
	}

	let { mode }: Props = $props();
</script>

<div class="text-sm text-surface-600-300-token">
	Operation mode changed to {mode.toUpperCase()}
</div>
