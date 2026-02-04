<script lang="ts">
	import '../app.css';
	import { AppShell, AppBar } from '@skeletonlabs/skeleton';
	import { onMount, onDestroy } from 'svelte';
	import { connectWebSocket, disconnectWebSocket } from '$lib/stores/websocket';
	import { Navigation, ThemeToggle, ConnectionStatus } from '$lib/components/layout';
	import { ToastContainer, ErrorBoundary } from '$lib/components/ui';

	onMount(() => {
		connectWebSocket();
	});

	onDestroy(() => {
		disconnectWebSocket();
	});
</script>

<ErrorBoundary>
	<div class="relative min-h-screen overflow-hidden">
		<div class="pointer-events-none absolute inset-0 bg-ember-radial-soft dark:bg-ember-radial"></div>
		<div class="pointer-events-none absolute inset-0 bg-cool-radial-soft dark:bg-cool-radial"></div>
		<div class="pointer-events-none absolute inset-0 bg-deep-radial-soft dark:bg-deep-radial"></div>

		<div class="relative">
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
		</div>
	</div>

	<ToastContainer />
</ErrorBoundary>
