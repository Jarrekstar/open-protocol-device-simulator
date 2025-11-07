<script lang="ts">
	interface Props {
		variant?: 'text' | 'circular' | 'rectangular' | 'card';
		width?: string;
		height?: string;
		count?: number;
		className?: string;
	}

	let {
		variant = 'rectangular',
		width = '100%',
		height = variant === 'text' ? '1rem' : '2rem',
		count = 1,
		className = ''
	}: Props = $props();

	const baseClasses = 'loading-skeleton';

	const variantClasses = {
		text: 'h-4',
		circular: 'rounded-full',
		rectangular: 'rounded-lg',
		card: 'rounded-lg h-32'
	};

	const skeletonClass = `${baseClasses} ${variantClasses[variant]} ${className}`;
</script>

{#if count > 1}
	<div class="space-y-3">
		{#each Array(count) as _, i (i)}
			<div
				class={skeletonClass}
				style="width: {width}; height: {height}"
			></div>
		{/each}
	</div>
{:else}
	<div
		class={skeletonClass}
		style="width: {width}; height: {height}"
	></div>
{/if}
