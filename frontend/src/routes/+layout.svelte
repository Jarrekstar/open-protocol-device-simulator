<script lang="ts">
	import '../app.css';
	import { AppShell, AppBar } from '@skeletonlabs/skeleton';
	import { onMount, onDestroy } from 'svelte';
	import { connectWebSocket, disconnectWebSocket } from '$lib/stores/websocket';
	import { Navigation, ThemeToggle, ConnectionStatus } from '$lib/components/layout';

	onMount(() => {
		connectWebSocket();
	});

	onDestroy(() => {
		disconnectWebSocket();
	});
</script>

<AppShell>
	<svelte:fragment slot="header">
		<AppBar>
			<svelte:fragment slot="lead">
				<div class="flex items-center gap-3">
					<img src="/logo.svg" alt="Device Simulator Logo" class="h-10 w-10" />
					<span class="text-xl uppercase font-medium">Device Simulator</span>
				</div>
			</svelte:fragment>
			<svelte:fragment slot="trail">
				<Navigation />
				<ThemeToggle />
				<ConnectionStatus />
			</svelte:fragment>
		</AppBar>
	</svelte:fragment>

	<div class="container mx-auto p-4">
		<slot />
	</div>
</AppShell>
