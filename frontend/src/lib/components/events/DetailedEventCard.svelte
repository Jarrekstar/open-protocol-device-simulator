<script lang="ts">
	import type { SimulatorEvent } from '$lib/types';
	import { Badge } from '../ui';
	import { formatTorque, formatAngle, formatBatchCounter } from '$lib/utils';

	interface Props {
		event: SimulatorEvent;
		index: number;
	}

	let { event, index }: Props = $props();
</script>

<article
	class="border-b border-surface-200-700-token px-5 py-4 even:bg-surface-100-800-token last:border-b-0"
>
	<header class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
		<div class="flex items-center gap-3">
			<Badge variant="filled-primary">{event.type}</Badge>
			<span class="text-xs uppercase tracking-wide text-surface-600-300-token"
				>Event #{index + 1}</span
			>
		</div>
	</header>

	{#if event.type === 'TighteningCompleted'}
		<dl class="mt-3 grid grid-cols-2 gap-3 text-sm md:grid-cols-4">
			<div>
				<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Status</dt>
				<dd class="mt-1">
					<Badge
						variant={event.result.tightening_status ? 'filled-success' : 'filled-error'}
					>
						{event.result.tightening_status ? 'OK' : 'NOK'}
					</Badge>
				</dd>
			</div>
			<div>
				<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Torque</dt>
				<dd class="mt-1 font-semibold">{formatTorque(event.result.torque)}</dd>
			</div>
			<div>
				<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Angle</dt>
				<dd class="mt-1 font-semibold">{formatAngle(event.result.angle)}</dd>
			</div>
			<div>
				<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Batch</dt>
				<dd class="mt-1 font-semibold">
					{formatBatchCounter(event.result.batch_counter, event.result.batch_size)}
				</dd>
			</div>
		</dl>
	{:else if event.type === 'MultiSpindleResultCompleted'}
		<dl class="mt-3 grid grid-cols-2 gap-3 text-sm md:grid-cols-4">
			<div>
				<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Status</dt>
				<dd class="mt-1">
					<Badge
						variant={event.result.overall_status === 0 ? 'filled-success' : 'filled-error'}
					>
						{event.result.overall_status === 0 ? 'OK' : 'NOK'}
					</Badge>
				</dd>
			</div>
			<div>
				<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Spindles</dt>
				<dd class="mt-1 font-semibold">{event.result.spindle_count}</dd>
			</div>
			<div>
				<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Sync ID</dt>
				<dd class="mt-1 font-semibold">{event.result.sync_id}</dd>
			</div>
			<div>
				<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Result ID</dt>
				<dd class="mt-1 font-semibold">{event.result.result_id}</dd>
			</div>
		</dl>
	{:else if event.type === 'BatchCompleted'}
		<p class="mt-3 text-sm text-surface-600-300-token">Batch total: {event.total}</p>
	{:else if event.type === 'ToolStateChanged'}
		<p class="mt-3 text-sm text-surface-600-300-token">
			Tool {event.enabled ? 'enabled' : 'disabled'}
		</p>
	{:else if event.type === 'PsetChanged'}
		<p class="mt-3 text-sm text-surface-600-300-token">
			PSET switched to {event.pset_name} (ID {event.pset_id})
		</p>
	{:else if event.type === 'VehicleIdChanged'}
		<p class="mt-3 text-sm text-surface-600-300-token">VIN updated to {event.vin}</p>
	{:else if event.type === 'MultiSpindleStatusCompleted'}
		<p class="mt-3 text-sm text-surface-600-300-token">
			Status: {event.status.status === 0
				? 'Waiting'
				: event.status.status === 1
					? 'Running'
					: 'Completed'}
			· Sync ID {event.status.sync_id} · Spindles {event.status.spindle_count}
		</p>
	{:else if event.type === 'AutoTighteningProgress'}
		<p class="mt-3 text-sm text-surface-600-300-token">
			Auto-tightening {event.running ? 'in progress' : 'stopped'} · {formatBatchCounter(
				event.counter,
				event.target_size
			)}
		</p>
	{/if}
</article>
