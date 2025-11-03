<script lang="ts">
	/**
	 * Badge component with variants
	 */
	interface Props {
		/** Badge variant */
		variant?:
			| 'soft'
			| 'filled'
			| 'filled-primary'
			| 'filled-success'
			| 'filled-error'
			| 'filled-warning';
		/** Additional CSS classes */
		class?: string;
	}

	let { variant = 'soft', class: className = '', children }: Props = $props();

	const variantClasses = {
		soft: 'badge variant-soft',
		filled: 'badge variant-filled',
		'filled-primary': 'badge variant-filled-primary',
		'filled-success': 'badge variant-filled-success',
		'filled-error': 'badge variant-filled-error',
		'filled-warning': 'badge variant-filled-warning'
	};
</script>

<span class="{variantClasses[variant]} {className}">
	{@render children()}
</span>
