<script lang="ts">
	import { api } from '$lib/api/client';
	import { deviceState } from '$lib/stores/device';
	import { showToast } from '$lib/stores/ui';
	import { formatErrorMessage, logger } from '$lib/utils';
	import type { Pset } from '$lib/types';
	import {
		PsetSelector,
		SingleTighteningForm,
		AutoTighteningPanel,
		MultiSpindleConfig,
		FailureInjectionPanel
	} from '$lib/components/control';
	import { onMount } from 'svelte';
	import ControlTabs from '$lib/components/ui/ControlTabs.svelte';
	import DataCard from '$lib/components/ui/DataCard.svelte';

	let psets: Pset[] = $state([]);
	let activeTab = $state('tightening');
	let isLoadingPsets = $state(false);
	let psetLoadError: Error | null = $state(null);

	const currentPset = $derived(psets.find((p) => p.id === $deviceState?.current_pset_id));

	const tabs = [
		{ id: 'tightening', label: 'Tightening', icon: '⚙️' },
		{ id: 'automation', label: 'Automation', icon: '🤖' },
		{ id: 'advanced', label: 'Advanced', icon: '🔧' }
	];

	async function loadPsets() {
		isLoadingPsets = true;
		psetLoadError = null;
		try {
			psets = await api.getPsets();
		} catch (error) {
			psetLoadError = error instanceof Error ? error : new Error(String(error));
			logger.error('Failed to load PSETs:', error);
			showToast({ type: 'error', message: formatErrorMessage('load PSETs', error) });
		} finally {
			isLoadingPsets = false;
		}
	}

	function handleTabChange(tabId: string) {
		activeTab = tabId;
	}

	onMount(() => {
		loadPsets();
	});
</script>

<svelte:head>
	<title>Control Panel - Device Simulator</title>
</svelte:head>

<div class="space-y-6 animate-fade-in">
	<!-- Header -->
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-3xl font-semibold text-surface-900 dark:text-surface-100">Control Panel</h1>
			<p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
				Configure tightening parameters and automation settings
			</p>
		</div>
		{#if currentPset}
			<div class="glass rounded-lg px-4 py-2">
				<div class="text-xs text-surface-600 dark:text-surface-400">Active PSET</div>
				<div class="text-sm font-semibold text-surface-900 dark:text-surface-100 mt-0.5">
					{currentPset.name}
				</div>
			</div>
		{/if}
	</div>

	<!-- Tabbed Interface -->
	<div class="card p-0 overflow-hidden">
		<ControlTabs {tabs} bind:activeTab onChange={handleTabChange} className="px-6 pt-6" />

		<div class="p-6">
			<!-- Tightening Tab -->
			{#if activeTab === 'tightening'}
				<div class="space-y-6 animate-slide-up">
					<div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
						<div class="lg:col-span-1">
							{#if isLoadingPsets}
								<div class="card p-6 text-center">
									<div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary-500"></div>
									<p class="text-sm text-surface-600 dark:text-surface-400 mt-3">Loading PSETs...</p>
								</div>
							{:else if psetLoadError}
								<div class="card p-6 border-error-500 bg-error-50 dark:bg-error-900/20">
									<div class="space-y-4">
										<div class="flex items-center gap-2 text-error-700 dark:text-error-300">
											<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
											</svg>
											<span class="font-semibold">Failed to load PSETs</span>
										</div>
										<p class="text-sm text-surface-700 dark:text-surface-300">{psetLoadError.message}</p>
										<button class="btn variant-filled-primary w-full" onclick={loadPsets}>
											Retry
										</button>
									</div>
								</div>
							{:else}
								<PsetSelector {psets} />
							{/if}
						</div>
						<div class="lg:col-span-2">
							<SingleTighteningForm {currentPset} />
						</div>
					</div>
				</div>
			{/if}

			<!-- Automation Tab -->
			{#if activeTab === 'automation'}
				<div class="space-y-6 animate-slide-up">
					<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
						<AutoTighteningPanel />
						<MultiSpindleConfig />
					</div>
				</div>
			{/if}

			<!-- Advanced Tab -->
			{#if activeTab === 'advanced'}
				<div class="animate-slide-up">
					<FailureInjectionPanel />
				</div>
			{/if}
		</div>
	</div>

	<!-- Quick Tips -->
	<DataCard
		title="💡 Quick Tips"
		subtitle="Optimize your workflow"
		padding="md"
		className="glass"
	>
		<div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
			<div class="space-y-1">
				<div class="font-medium text-surface-900 dark:text-surface-100">PSET Mode</div>
				<p class="text-surface-600 dark:text-surface-400">
					Select a PSET to use predefined torque and angle values
				</p>
			</div>
			<div class="space-y-1">
				<div class="font-medium text-surface-900 dark:text-surface-100">Auto-Tightening</div>
				<p class="text-surface-600 dark:text-surface-400">
					Automate repetitive tightening cycles with configurable intervals
				</p>
			</div>
			<div class="space-y-1">
				<div class="font-medium text-surface-900 dark:text-surface-100">Failure Injection</div>
				<p class="text-surface-600 dark:text-surface-400">
					Simulate connection issues to test error handling
				</p>
			</div>
		</div>
	</DataCard>
</div>
