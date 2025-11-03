<script lang="ts">
	import type { TighteningResult } from '$lib/types';
	import { formatTorque, formatAngle, formatBatchCounter } from '$lib/utils';

	interface Props {
		result: TighteningResult;
	}

	let { result }: Props = $props();
</script>

<div class="text-sm text-surface-600-300-token">
	{formatTorque(result.torque)} · {formatAngle(result.angle)} ·
	Batch {formatBatchCounter(result.batch_counter, result.batch_size)}
</div>
