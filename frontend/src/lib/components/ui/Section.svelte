<script lang="ts">
	import Card from './Card.svelte';

	/**
	 * Section component - Card with header and optional description
	 */
	interface Props {
		/** Section title */
		title: string;
		/** Optional description text */
		description?: string;
		/** Additional CSS classes */
		class?: string;
		/** Padding variant */
		padding?: 'sm' | 'md' | 'lg' | 'none';
		/** Spacing between children */
		spacing?: 'sm' | 'md' | 'lg' | 'none';
		/** Header action slot */
		headerAction?: any;
	}

	let {
		title,
		description,
		class: className = '',
		padding = 'md',
		spacing = 'md',
		headerAction,
		children
	}: Props = $props();
</script>

<Card {padding} {spacing} class={className}>
	<div class="flex items-center justify-between">
		<div>
			<h2 class="h2">{title}</h2>
			{#if description}
				<p class="text-sm opacity-70">{description}</p>
			{/if}
		</div>
		{#if headerAction}
			{@render headerAction()}
		{/if}
	</div>
	{@render children()}
</Card>
