<script lang="ts">
	interface Tab {
		id: string;
		label: string;
		icon?: string;
		badge?: string | number;
	}

	interface Props {
		tabs: Tab[];
		activeTab: string;
		onChange: (tabId: string) => void;
		className?: string;
	}

	let { tabs, activeTab = $bindable(), onChange, className = '' }: Props = $props();

	function handleTabClick(tabId: string) {
		activeTab = tabId;
		onChange(tabId);
	}
</script>

<div class="border-b border-surface-300 dark:border-surface-400 {className}">
	<nav class="-mb-px flex space-x-2 overflow-x-auto" aria-label="Tabs">
		{#each tabs as tab}
			<button
				type="button"
				class="group inline-flex items-center gap-2 px-4 py-3 border-b-2 font-medium text-sm whitespace-nowrap transition-all duration-200
					{activeTab === tab.id
					? 'border-primary-500 dark:border-primary-400 text-primary-600 dark:text-primary-400'
					: 'border-transparent text-surface-600 dark:text-surface-400 hover:text-surface-900 dark:hover:text-surface-100 hover:border-surface-300 dark:hover:border-surface-500'}"
				onclick={() => handleTabClick(tab.id)}
				aria-current={activeTab === tab.id ? 'page' : undefined}
			>
				{#if tab.icon}
					<span class="text-lg">{tab.icon}</span>
				{/if}
				{tab.label}
				{#if tab.badge !== undefined}
					<span
						class="ml-1 rounded-full px-2 py-0.5 text-xs font-medium
						{activeTab === tab.id
							? 'bg-primary-100 dark:bg-primary-900 text-primary-700 dark:text-primary-300'
							: 'bg-surface-200 dark:bg-surface-300 text-surface-700 dark:text-surface-200'}"
					>
						{tab.badge}
					</span>
				{/if}
			</button>
		{/each}
	</nav>
</div>
