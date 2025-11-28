<script lang="ts">
	import { page } from '$app/state';

	const navItems = [
		{ href: '/', label: 'Dashboard' },
		{ href: '/control', label: 'Control Panel' },
		{ href: '/psets', label: 'PSETs' },
		{ href: '/events', label: 'Events' }
	];
</script>

<nav class="flex gap-4">
	{#each navItems as item}
		<a
			href={item.href}
			class="btn btn-sm"
			class:variant-filled-primary={page.url.pathname === item.href}
			class:variant-ghost-surface={page.url.pathname !== item.href}
		>
			{item.label}
		</a>
	{/each}
</nav>
