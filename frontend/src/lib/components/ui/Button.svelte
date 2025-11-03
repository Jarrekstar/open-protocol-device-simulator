<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * Button component with variants
	 */
	interface Props {
		/** Button variant */
		variant?:
			| 'filled-primary'
			| 'filled-secondary'
			| 'filled-error'
			| 'filled-success'
			| 'ghost-surface';
		/** Button type */
		type?: 'button' | 'submit' | 'reset';
		/** Disabled state */
		disabled?: boolean;
		/** Additional CSS classes */
		class?: string;
		/** Click handler */
		onclick?: (event: MouseEvent) => void;
		/** Button content */
		children: Snippet;
	}

	let {
		variant = 'filled-primary',
		type = 'button',
		disabled = false,
		class: className = '',
		onclick,
		children
	}: Props = $props();

	const variantClasses = {
		'filled-primary': 'btn variant-filled-primary',
		'filled-secondary': 'btn variant-filled-secondary',
		'filled-error': 'btn variant-filled-error',
		'filled-success': 'btn variant-filled-success',
		'ghost-surface': 'btn variant-ghost-surface'
	};
</script>

<button
	{type}
	{disabled}
	class="{variantClasses[variant]} {className}"
	{onclick}
>
	{@render children()}
</button>
