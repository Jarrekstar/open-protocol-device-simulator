<script lang="ts">
	interface Props {
		status: 'connected' | 'disconnected' | 'pending' | 'error' | 'success' | 'warning';
		label?: string;
		showLabel?: boolean;
		size?: 'sm' | 'md' | 'lg';
		pulse?: boolean;
	}

	let {
		status,
		label = '',
		showLabel = true,
		size = 'md',
		pulse = false
	}: Props = $props();

	const sizeClasses = {
		sm: 'w-2 h-2',
		md: 'w-3 h-3',
		lg: 'w-4 h-4'
	};

	const statusColors = {
		connected: 'bg-success-500',
		success: 'bg-success-500',
		disconnected: 'bg-error-500',
		error: 'bg-error-500',
		pending: 'bg-warning-500',
		warning: 'bg-warning-500'
	};

	const statusLabels = {
		connected: 'Connected',
		disconnected: 'Disconnected',
		pending: 'Pending',
		error: 'Error',
		success: 'Success',
		warning: 'Warning'
	};

	const displayLabel = $derived(label || statusLabels[status]);
</script>

<div
	class="inline-flex items-center gap-2"
	role="status"
	aria-label={displayLabel}
>
	<span class="relative flex {sizeClasses[size]}">
		<span
			class="absolute inline-flex h-full w-full rounded-full {statusColors[status]} opacity-75 {pulse
				? 'animate-ping'
				: ''}"
		></span>
		<span class="relative inline-flex rounded-full {sizeClasses[size]} {statusColors[status]}"></span>
	</span>
	{#if showLabel}
		<span class="text-sm font-medium text-surface-800 dark:text-surface-900">
			{displayLabel}
		</span>
	{/if}
</div>
