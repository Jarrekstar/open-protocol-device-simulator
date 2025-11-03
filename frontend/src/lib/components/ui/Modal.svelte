<script lang="ts">
	import { fade, scale } from 'svelte/transition';
	import type { Snippet } from 'svelte';

	/**
	 * Modal component with backdrop and transitions
	 */
	interface Props {
		/** Whether the modal is open */
		open: boolean;
		/** Close handler */
		onclose?: () => void;
		/** Modal title */
		title?: string;
		/** Modal description */
		description?: string;
		/** Additional CSS classes for content */
		class?: string;
		/** Max width */
		maxWidth?: 'sm' | 'md' | 'lg' | 'xl';
		/** Modal content */
		children: Snippet;
	}

	let { open, onclose, title, description, class: className = '', maxWidth = 'md', children }: Props =
		$props();

	const maxWidthClasses = {
		sm: 'max-w-md',
		md: 'max-w-2xl',
		lg: 'max-w-4xl',
		xl: 'max-w-6xl'
	};

	function handleBackdropClick() {
		onclose?.();
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="modal-backdrop"
		onclick={handleBackdropClick}
		transition:fade={{ duration: 150 }}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="modal-content rounded-2xl border border-surface-200-700-token bg-surface-100-800-token p-6 shadow-2xl {maxWidthClasses[maxWidth]} {className}"
			onclick={(e) => e.stopPropagation()}
			transition:scale={{ duration: 200, start: 0.95 }}
		>
			{#if title}
				<header class="mb-4">
					<h2 class="h2">{title}</h2>
					{#if description}
						<p class="text-sm opacity-70">{description}</p>
					{/if}
				</header>
			{/if}

			{@render children()}
		</div>
	</div>
{/if}
