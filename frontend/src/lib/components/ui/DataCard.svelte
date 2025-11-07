<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title?: string;
		subtitle?: string;
		headerAction?: Snippet;
		footer?: Snippet;
		children?: Snippet;
		padding?: 'none' | 'sm' | 'md' | 'lg';
		hover?: boolean;
		accent?: boolean;
		className?: string;
	}

	let {
		title,
		subtitle,
		headerAction,
		footer,
		children,
		padding = 'md',
		hover = true,
		accent = false,
		className = ''
	}: Props = $props();

	const paddingClasses = {
		none: '',
		sm: 'p-4',
		md: 'p-6',
		lg: 'p-8'
	};
</script>

<div
	class="card {accent ? 'metric-card' : ''} {hover ? 'hover-lift' : ''} {paddingClasses[
		padding
	]} {className}"
>
	{#if title || subtitle || headerAction}
		<div class="flex items-start justify-between mb-4 {padding === 'none' ? 'px-6 pt-6' : ''}">
			<div class="flex-1">
				{#if title}
					<h3 class="text-lg font-medium text-surface-900 dark:text-surface-100">
						{title}
					</h3>
				{/if}
				{#if subtitle}
					<p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
						{subtitle}
					</p>
				{/if}
			</div>
			{#if headerAction}
				<div class="ml-4">
					{@render headerAction()}
				</div>
			{/if}
		</div>
	{/if}

	{#if children}
		<div class={padding === 'none' ? 'px-6' : ''}>
			{@render children()}
		</div>
	{/if}

	{#if footer}
		<div
			class="mt-4 pt-4 border-t border-surface-300 dark:border-surface-400 {padding === 'none'
				? 'px-6 pb-6'
				: ''}"
		>
			{@render footer()}
		</div>
	{/if}
</div>
