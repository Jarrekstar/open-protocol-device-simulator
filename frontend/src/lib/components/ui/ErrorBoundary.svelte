<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { browser } from '$app/environment';
	import type { Snippet } from 'svelte';
	import { logger } from '$lib/utils';

	interface Props {
		/** Content to render */
		children: Snippet;
		/** Fallback content when error occurs (optional) */
		fallback?: Snippet<[Error]>;
	}

	let { children, fallback }: Props = $props();

	let error = $state<Error | null>(null);

	// Global error handlers (only in browser)
	function handleError(event: ErrorEvent) {
		event.preventDefault();
		error = event.error instanceof Error ? event.error : new Error(event.message);
		logger.error('Global error caught:', error);
	}

	function handleUnhandledRejection(event: PromiseRejectionEvent) {
		event.preventDefault();
		error = event.reason instanceof Error ? event.reason : new Error(String(event.reason));
		logger.error('Unhandled promise rejection:', error);
	}

	onMount(() => {
		if (browser) {
			window.addEventListener('error', handleError);
			window.addEventListener('unhandledrejection', handleUnhandledRejection);
		}
	});

	onDestroy(() => {
		if (browser) {
			window.removeEventListener('error', handleError);
			window.removeEventListener('unhandledrejection', handleUnhandledRejection);
		}
	});

	function reset() {
		error = null;
	}
</script>

{#if error}
	{#if fallback}
		{@render fallback(error)}
	{:else}
		<div class="card p-6 border-error-500 bg-error-50 dark:bg-error-900/20">
			<div class="space-y-4">
				<div class="flex items-center gap-3">
					<svg
						class="w-6 h-6 text-error-500"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
						/>
					</svg>
					<h2 class="h3 text-error-700 dark:text-error-300">Something went wrong</h2>
				</div>

				<p class="text-sm opacity-80">
					An unexpected error occurred. Please try refreshing the page or contact support if the
					problem persists.
				</p>

				<details class="text-xs opacity-70">
					<summary class="cursor-pointer hover:opacity-100">Error details</summary>
					<pre
						class="mt-2 p-3 rounded bg-surface-900 text-surface-50 overflow-auto max-h-40">{error.message}\n\n{error.stack}</pre>
				</details>

				<button class="btn variant-filled-error" onclick={reset}> Try Again </button>
			</div>
		</div>
	{/if}
{:else}
	{@render children()}
{/if}
