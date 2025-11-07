<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		description?: string;
		icon?: string;
		action?: Snippet;
		children?: Snippet;
	}

	let { title, description, icon, action, children }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center py-12 px-4 text-center">
	{#if icon}
		<div class="mb-4 text-6xl opacity-30">
			{icon}
		</div>
	{:else}
		<div class="mb-4 h-16 w-16 rounded-full bg-surface-200 dark:bg-surface-300 flex items-center justify-center">
			<svg
				class="w-8 h-8 text-surface-500"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"
				/>
			</svg>
		</div>
	{/if}

	<h3 class="text-xl font-medium text-surface-900 dark:text-surface-100 mb-2">
		{title}
	</h3>

	{#if description}
		<p class="text-sm text-surface-600 dark:text-surface-400 max-w-sm mb-6">
			{description}
		</p>
	{/if}

	{#if action}
		<div class="mt-4">
			{@render action()}
		</div>
	{/if}

	{#if children}
		<div class="mt-6 w-full">
			{@render children()}
		</div>
	{/if}
</div>
