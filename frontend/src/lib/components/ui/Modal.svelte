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

	let modalElement: HTMLDivElement | undefined = $state();
	let previousActiveElement: Element | null = null;

	// Handle ESC key and focus trap
	$effect(() => {
		if (open) {
			// Store the currently focused element
			previousActiveElement = document.activeElement;

			// Focus the modal content
			if (modalElement) {
				modalElement.focus();
			}

			// ESC key handler
			const handleKeyDown = (e: KeyboardEvent) => {
				if (e.key === 'Escape') {
					onclose?.();
				}

				// Focus trap
				if (e.key === 'Tab' && modalElement) {
					const focusableElements = modalElement.querySelectorAll<HTMLElement>(
						'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
					);
					const firstElement = focusableElements[0];
					const lastElement = focusableElements[focusableElements.length - 1];

					if (e.shiftKey && document.activeElement === firstElement) {
						e.preventDefault();
						lastElement?.focus();
					} else if (!e.shiftKey && document.activeElement === lastElement) {
						e.preventDefault();
						firstElement?.focus();
					}
				}
			};

			document.addEventListener('keydown', handleKeyDown);

			// Cleanup
			return () => {
				document.removeEventListener('keydown', handleKeyDown);
				// Restore focus to previous element
				if (previousActiveElement instanceof HTMLElement) {
					previousActiveElement.focus();
				}
			};
		}
	});

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
		<div
			bind:this={modalElement}
			role="dialog"
			aria-modal="true"
			aria-labelledby={title ? 'modal-title' : undefined}
			aria-describedby={description ? 'modal-description' : undefined}
			tabindex="-1"
			class="modal-content rounded-2xl border border-surface-200-700-token bg-surface-100-800-token p-6 shadow-2xl {maxWidthClasses[maxWidth]} {className}"
			onclick={(e) => e.stopPropagation()}
			transition:scale={{ duration: 200, start: 0.95 }}
		>
			{#if title}
				<header class="mb-4">
					<h2 id="modal-title" class="h2">{title}</h2>
					{#if description}
						<p id="modal-description" class="text-sm opacity-70">{description}</p>
					{/if}
				</header>
			{/if}

			{@render children()}
		</div>
	</div>
{/if}
