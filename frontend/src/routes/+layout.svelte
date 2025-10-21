<script lang="ts">
	import '../app.css';
	import { AppShell, AppBar } from '@skeletonlabs/skeleton';
	import { onMount, onDestroy } from 'svelte';
	import { connectWebSocket, disconnectWebSocket, connected } from '$lib/stores/websocket';
	import { page } from '$app/stores';
	import { theme, toggleTheme } from '$lib/stores/theme';

	onMount(() => {
		connectWebSocket();
	});

	onDestroy(() => {
		disconnectWebSocket();
	});

	const navItems = [
		{ href: '/', label: 'Dashboard' },
		{ href: '/control', label: 'Control Panel' },
		{ href: '/psets', label: 'PSETs' },
		{ href: '/events', label: 'Events' }
	];
</script>

<AppShell>
	<svelte:fragment slot="header">
		<AppBar>
			<svelte:fragment slot="lead">
				<span class="text-xl uppercase font-medium">Device Simulator</span>
			</svelte:fragment>
			<svelte:fragment slot="trail">
				<nav class="flex gap-4">
					{#each navItems as item}
						<a
							href={item.href}
							class="btn btn-sm"
							class:variant-filled-primary={$page.url.pathname === item.href}
							class:variant-ghost-surface={$page.url.pathname !== item.href}
						>
							{item.label}
						</a>
					{/each}
				</nav>
				<button
					class="btn btn-sm variant-ghost-surface"
					on:click={toggleTheme}
					title="Toggle theme"
				>
					{#if $theme === 'light'}
						🌙
					{:else}
						☀️
					{/if}
				</button>
				<div class="flex items-center gap-2">
					<div
						class="w-2 h-2 rounded-full"
						class:bg-success-500={$connected}
						class:bg-error-500={!$connected}
					/>
					<span class="text-sm">{$connected ? 'Connected' : 'Disconnected'}</span>
				</div>
			</svelte:fragment>
		</AppBar>
	</svelte:fragment>

	<div class="container mx-auto p-4">
		<slot />
	</div>
</AppShell>

<style>
	:global(.app-bar.bg-surface-100-800-token) {
		background: rgba(212, 196, 176, 0.3) !important;
		backdrop-filter: blur(10px);
		border: none !important;
		border-bottom: 1px solid rgba(200, 180, 160, 0.3) !important;
		box-shadow: none !important;
		color: #2c2416 !important;
	}

	:global(.dark .app-bar.bg-surface-100-800-token) {
		background: rgba(29, 56, 50, 0.3) !important;
		border-bottom: 1px solid rgba(42, 82, 72, 0.3) !important;
		color: #d4e8e3 !important;
	}

	/* Fix AppBar button colors in dark mode */
	:global(.dark .app-bar .btn.variant-ghost-surface) {
		border-color: #253d35 !important;
		color: #9bc4ba !important;
	}

	:global(.dark .app-bar .btn.variant-ghost-surface:hover) {
		border-color: #2a5248 !important;
		background: #1a2e29 !important;
	}

	:global(.dark .app-bar .btn.variant-filled-primary) {
		background: linear-gradient(135deg, #2a5248 0%, #234439 100%) !important;
		border: none !important;
		color: white !important;
	}

	:global(.dark .app-bar .btn.variant-filled-primary:hover) {
		background: linear-gradient(135deg, #326156 0%, #2a5248 100%) !important;
	}
</style>
