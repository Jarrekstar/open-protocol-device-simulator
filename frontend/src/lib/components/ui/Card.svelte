<script lang="ts">
	/**
	 * Reusable card component with consistent styling
	 */
	interface Props {
		/** Additional CSS classes */
		class?: string;
		/** Padding variant */
		padding?: 'sm' | 'md' | 'lg' | 'none';
		/** Spacing between children */
		spacing?: 'sm' | 'md' | 'lg' | 'none';
	}

	let { class: className = '', padding = 'md', spacing = 'md', children }: Props = $props();

	const paddingClasses = {
		none: '',
		sm: 'p-4',
		md: 'p-4 sm:p-6',
		lg: 'p-6 sm:p-8'
	};

	const spacingClasses = {
		none: '',
		sm: 'space-y-3',
		md: 'space-y-4',
		lg: 'space-y-6'
	};
</script>

<section class="card {paddingClasses[padding]} {spacingClasses[spacing]} {className}">
	{@render children()}
</section>
