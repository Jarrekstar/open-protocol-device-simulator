<script lang="ts">
	interface Mode {
		id: string;
		label: string;
		icon?: string;
	}

	interface Props {
		modes: Mode[];
		activeMode: string;
		onChange: (modeId: string) => void;
		size?: 'sm' | 'md' | 'lg';
		className?: string;
	}

	let { modes, activeMode = $bindable(), onChange, size = 'md', className = '' }: Props = $props();

	const sizeClasses = {
		sm: 'text-xs px-2 py-1',
		md: 'text-sm px-3 py-1.5',
		lg: 'text-base px-4 py-2'
	};

	function handleModeClick(modeId: string) {
		activeMode = modeId;
		onChange(modeId);
	}
</script>

<div class="inline-flex items-center gap-1 rounded-lg bg-surface-200 dark:bg-surface-300 p-1 {className}">
	{#each modes as mode}
		<button
			type="button"
			class="rounded-md font-medium transition-all duration-200 {sizeClasses[size]}
				{activeMode === mode.id
				? 'bg-surface-50 dark:bg-surface-100 text-surface-900 shadow-sm'
				: 'text-surface-600 dark:text-surface-700 hover:text-surface-900 dark:hover:text-surface-900'}"
			onclick={() => handleModeClick(mode.id)}
			aria-pressed={activeMode === mode.id}
		>
			{#if mode.icon}
				<span class="mr-1">{mode.icon}</span>
			{/if}
			{mode.label}
		</button>
	{/each}
</div>
