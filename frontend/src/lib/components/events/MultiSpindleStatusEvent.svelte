<script lang="ts">
	import type { MultiSpindleStatus } from '$lib/types';

	interface Props {
		status: MultiSpindleStatus;
	}

	let { status }: Props = $props();
</script>

<div class="text-sm text-surface-600-300-token">
	Multi-spindle status · Sync {status.sync_id} · Spindles {status.spindle_count}
</div>
