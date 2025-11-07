<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/client';
	import { showToast } from '$lib/stores/ui';
	import { formatErrorMessage } from '$lib/utils';
	import { Button } from '$lib/components/ui';
	import { PsetModal, PsetCard } from '$lib/components/psets';
	import type { Pset } from '$lib/types';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import LoadingSkeleton from '$lib/components/ui/LoadingSkeleton.svelte';

	let psets: Pset[] = $state([]);
	let loading = $state(true);
	let showModal = $state(false);
	let modalMode: 'create' | 'edit' = $state('create');
	let editingPset: Pset | null = $state(null);
	let deleteConfirmId: number | null = $state(null);
	let searchQuery = $state('');

	const filteredPsets = $derived.by(() => {
		if (!searchQuery.trim()) return psets;
		const query = searchQuery.toLowerCase();
		return psets.filter(
			(p) =>
				p.name.toLowerCase().includes(query) ||
				p.description?.toLowerCase().includes(query) ||
				p.id.toString().includes(query)
		);
	});

	async function loadPsets() {
		try {
			loading = true;
			psets = await api.getPsets();
		} catch (error) {
			showToast({ type: 'error', message: formatErrorMessage('load PSETs', error) });
		} finally {
			loading = false;
		}
	}

	function openCreateModal() {
		modalMode = 'create';
		editingPset = null;
		showModal = true;
	}

	function openEditModal(pset: Pset) {
		modalMode = 'edit';
		editingPset = pset;
		showModal = true;
	}

	function closeModal() {
		showModal = false;
		editingPset = null;
	}

	async function handleSubmit(formData: Partial<Pset>) {
		try {
			if (modalMode === 'create') {
				await api.createPset({
					name: formData.name!,
					torque_min: formData.torque_min!,
					torque_max: formData.torque_max!,
					angle_min: formData.angle_min!,
					angle_max: formData.angle_max!,
					description: formData.description || undefined
				});
				showToast({ type: 'success', message: 'PSET created successfully!' });
			} else if (editingPset) {
				await api.updatePset(editingPset.id, {
					id: editingPset.id,
					name: formData.name!,
					torque_min: formData.torque_min!,
					torque_max: formData.torque_max!,
					angle_min: formData.angle_min!,
					angle_max: formData.angle_max!,
					description: formData.description || undefined
				});
				showToast({ type: 'success', message: 'PSET updated successfully!' });
			}

			closeModal();
			await loadPsets();
		} catch (error) {
			showToast({ type: 'error', message: formatErrorMessage('save PSET', error) });
			throw error; // Re-throw to keep form in submitting state
		}
	}

	async function handleDelete(id: number) {
		try {
			await api.deletePset(id);
			showToast({ type: 'success', message: 'PSET deleted successfully!' });
			deleteConfirmId = null;
			await loadPsets();
		} catch (error) {
			showToast({ type: 'error', message: formatErrorMessage('delete PSET', error) });
		}
	}

	onMount(() => {
		loadPsets();
	});
</script>

<svelte:head>
	<title>PSET Management - Device Simulator</title>
</svelte:head>

<div class="space-y-6 animate-fade-in">
	<!-- Header -->
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-3xl font-semibold text-surface-900 dark:text-surface-100">
				Parameter Sets
			</h1>
			<p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
				Reusable torque and angle configurations for tightening operations
			</p>
		</div>
		<Button onclick={openCreateModal} variant="filled-primary">
			<span>+ Create PSET</span>
		</Button>
	</div>

	<!-- Search & Filters -->
	{#if psets.length > 0}
		<div class="relative">
			<svg
				class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-surface-400"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
				/>
			</svg>
			<input
				type="search"
				class="input !pl-11 w-full max-w-md"
				placeholder="Search PSETs by name, ID, or description..."
				bind:value={searchQuery}
			/>
		</div>
	{/if}

	<!-- PSET Grid -->
	{#if loading}
		<div class="grid auto-rows-fr grid-cols-1 gap-6 sm:grid-cols-2 xl:grid-cols-3">
			{#each Array(6) as _, i (i)}
				<div class="card p-6">
					<LoadingSkeleton variant="rectangular" height="200px" />
				</div>
			{/each}
		</div>
	{:else if psets.length === 0}
		<EmptyState
			title="No PSETs Created Yet"
			description="Parameter Sets allow you to define reusable tightening configurations with specific torque and angle ranges."
			icon="⚙️"
		>
			{#snippet action()}
				<Button onclick={openCreateModal} variant="filled-primary">
					Create Your First PSET
				</Button>
			{/snippet}
		</EmptyState>
	{:else if filteredPsets.length === 0}
		<EmptyState
			title="No PSETs Match Your Search"
			description='Try adjusting your search query or clear it to see all PSETs.'
			icon="🔍"
		>
			{#snippet action()}
				<Button onclick={() => (searchQuery = '')} variant="filled-primary"> Clear Search </Button>
			{/snippet}
		</EmptyState>
	{:else}
		<!-- PSET Count -->
		<div class="flex items-center justify-between">
			<p class="text-sm text-surface-600 dark:text-surface-400">
				{filteredPsets.length === psets.length
					? `${psets.length} ${psets.length === 1 ? 'PSET' : 'PSETs'}`
					: `Showing ${filteredPsets.length} of ${psets.length} PSETs`}
			</p>
		</div>

		<!-- Grid -->
		<div class="grid auto-rows-fr grid-cols-1 gap-6 sm:grid-cols-2 xl:grid-cols-3">
			{#each filteredPsets as pset, i (pset.id)}
				<div class="animate-slide-up" style="animation-delay: {i * 0.05}s">
					<PsetCard
						{pset}
						onEdit={openEditModal}
						onDelete={handleDelete}
						{deleteConfirmId}
						onToggleDeleteConfirm={(id) => (deleteConfirmId = id)}
					/>
				</div>
			{/each}
		</div>
	{/if}
</div>

<PsetModal
	open={showModal}
	mode={modalMode}
	pset={editingPset}
	onsubmit={handleSubmit}
	onclose={closeModal}
/>
