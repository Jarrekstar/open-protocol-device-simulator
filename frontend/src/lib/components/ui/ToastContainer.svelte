<script lang="ts">
	import { toasts, dismissToast } from '$lib/stores/ui';
	import { fly } from 'svelte/transition';
	import { Badge } from './';

	const variantMap = {
		success: 'filled-success',
		error: 'filled-error',
		warning: 'filled-warning',
		info: 'soft'
	} as const;
</script>

<div class="fixed top-4 right-4 z-50 space-y-2 max-w-md">
	{#each $toasts as toast (toast.id)}
		<div
			class="card p-4 shadow-2xl flex items-start gap-3 border-l-4"
			class:border-success-500={toast.type === 'success'}
			class:border-error-500={toast.type === 'error'}
			class:border-warning-500={toast.type === 'warning'}
			class:border-primary-500={toast.type === 'info'}
			transition:fly={{ x: 300, duration: 300 }}
		>
			<div class="flex-1">
				<div class="flex items-center gap-2 mb-1">
					<Badge variant={variantMap[toast.type]} class="capitalize">
						{toast.type}
					</Badge>
				</div>
				<p class="text-sm">{toast.message}</p>
			</div>
			<button
				onclick={() => dismissToast(toast.id)}
				class="text-surface-600-300-token hover:text-surface-900 transition-colors"
				aria-label="Dismiss notification"
			>
				<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M6 18L18 6M6 6l12 12"
					/>
				</svg>
			</button>
		</div>
	{/each}
</div>
