<script lang="ts">
	import type { MultiSpindleResult } from '$lib/types';

	interface Props {
		result: MultiSpindleResult;
	}

	let { result }: Props = $props();
</script>

<div class="text-sm text-surface-600-300-token">
	Multi-spindle result · {result.spindle_count} spindles · Sync {result.sync_id}
</div>
