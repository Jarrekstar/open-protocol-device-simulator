<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/client';
	import { showToast } from '$lib/stores/ui';
	import { Button, Card } from '$lib/components/ui';
	import { PsetModal, PsetCard } from '$lib/components/psets';
	import type { Pset } from '$lib/types';

	let psets: Pset[] = $state([]);
	let loading = $state(true);
	let showModal = $state(false);
	let modalMode: 'create' | 'edit' = $state('create');
	let editingPset: Pset | null = $state(null);
	let deleteConfirmId: number | null = $state(null);

	async function loadPsets() {
		try {
			loading = true;
			psets = await api.getPsets();
		} catch (error) {
			showToast({ type: 'error', message: `Failed to load PSETs: ${error}` });
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
		} catch (error: any) {
			const errorMessage = error?.message || String(error);
			showToast({ type: 'error', message: `Failed to save PSET: ${errorMessage}` });
			throw error; // Re-throw to keep form in submitting state
		}
	}

	async function handleDelete(id: number) {
		try {
			await api.deletePset(id);
			showToast({ type: 'success', message: 'PSET deleted successfully!' });
			deleteConfirmId = null;
			await loadPsets();
		} catch (error: any) {
			const errorMessage = error?.message || String(error);
			showToast({ type: 'error', message: `Failed to delete PSET: ${errorMessage}` });
		}
	}

	onMount(() => {
		loadPsets();
	});
</script>

<svelte:head>
	<title>PSET Management - Device Simulator</title>
</svelte:head>

<div class="mx-auto max-w-6xl space-y-6">
	<header class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="h1">Parameter Sets (PSETs)</h1>
			<p class="text-sm opacity-70">
				Manage reusable torque and angle presets for quick selection.
			</p>
		</div>
		<Button onclick={openCreateModal} class="sm:w-auto">
			<span>+ Create New PSET</span>
		</Button>
	</header>

	{#if loading}
		<div class="flex h-64 items-center justify-center">
			<div class="text-lg text-surface-500">Loading PSETs...</div>
		</div>
	{:else if psets.length === 0}
		<Card padding="lg" class="text-center">
			<p class="text-lg text-surface-600-300-token">No PSETs available yet.</p>
			<p class="mt-2 text-sm opacity-70">
				Create one to define tightening targets for your scenarios.
			</p>
			<Button onclick={openCreateModal} class="mt-4">
				Create Your First PSET
			</Button>
		</Card>
	{:else}
		<div class="grid auto-rows-fr grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
			{#each psets as pset (pset.id)}
				<PsetCard
					{pset}
					onEdit={openEditModal}
					onDelete={handleDelete}
					{deleteConfirmId}
					onToggleDeleteConfirm={(id) => (deleteConfirmId = id)}
				/>
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
