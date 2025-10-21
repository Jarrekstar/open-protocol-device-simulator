<script lang="ts">
	import { onMount } from 'svelte';
	import { fade, scale } from 'svelte/transition';
	import { api } from '$lib/api/client';
	import { showToast } from '$lib/stores/ui';
	import type { Pset } from '$lib/types';

	let psets: Pset[] = [];
	let loading = true;
	let showModal = false;
	let modalMode: 'create' | 'edit' = 'create';
	let editingPset: Pset | null = null;
	let deleteConfirmId: number | null = null;

	// Form fields
	let formData = {
		name: '',
		torque_min: 10.0,
		torque_max: 15.0,
		angle_min: 30.0,
		angle_max: 45.0,
		description: ''
	};

	let formErrors = {
		name: '',
		torque: '',
		angle: ''
	};

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
		resetForm();
		showModal = true;
	}

	function openEditModal(pset: Pset) {
		modalMode = 'edit';
		editingPset = pset;
		formData = {
			name: pset.name,
			torque_min: pset.torque_min,
			torque_max: pset.torque_max,
			angle_min: pset.angle_min,
			angle_max: pset.angle_max,
			description: pset.description || ''
		};
		showModal = true;
	}

	function resetForm() {
		formData = {
			name: '',
			torque_min: 10.0,
			torque_max: 15.0,
			angle_min: 30.0,
			angle_max: 45.0,
			description: ''
		};
		formErrors = {
			name: '',
			torque: '',
			angle: ''
		};
	}

	function validateForm(): boolean {
		formErrors = {
			name: '',
			torque: '',
			angle: ''
		};

		let isValid = true;

		if (!formData.name.trim()) {
			formErrors.name = 'Name is required';
			isValid = false;
		}

		if (formData.torque_min >= formData.torque_max) {
			formErrors.torque = 'Min torque must be less than max torque';
			isValid = false;
		}

		if (formData.torque_min < 0) {
			formErrors.torque = 'Torque values must be non-negative';
			isValid = false;
		}

		if (formData.angle_min >= formData.angle_max) {
			formErrors.angle = 'Min angle must be less than max angle';
			isValid = false;
		}

		if (formData.angle_min < 0 || formData.angle_max > 360) {
			formErrors.angle = 'Angle must be between 0 and 360 degrees';
			isValid = false;
		}

		return isValid;
	}

	async function handleSubmit() {
		if (!validateForm()) {
			return;
		}

		try {
			if (modalMode === 'create') {
				await api.createPset({
					name: formData.name,
					torque_min: formData.torque_min,
					torque_max: formData.torque_max,
					angle_min: formData.angle_min,
					angle_max: formData.angle_max,
					description: formData.description || undefined
				});
				showToast({ type: 'success', message: 'PSET created successfully!' });
			} else if (editingPset) {
				await api.updatePset(editingPset.id, {
					id: editingPset.id,
					name: formData.name,
					torque_min: formData.torque_min,
					torque_max: formData.torque_max,
					angle_min: formData.angle_min,
					angle_max: formData.angle_max,
					description: formData.description || undefined
				});
				showToast({ type: 'success', message: 'PSET updated successfully!' });
			}

			showModal = false;
			resetForm();
			await loadPsets();
		} catch (error: any) {
			const errorMessage = error?.message || String(error);
			showToast({ type: 'error', message: `Failed to save PSET: ${errorMessage}` });
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

<div class="container mx-auto">
	<div class="flex justify-between items-center mb-6">
		<h1 class="h1">Parameter Sets (PSETs)</h1>
		<button class="btn variant-filled-primary" on:click={openCreateModal}>
			<span>+ Create New PSET</span>
		</button>
	</div>

	{#if loading}
		<div class="flex justify-center items-center h-64">
			<div class="text-lg">Loading PSETs...</div>
		</div>
	{:else if psets.length === 0}
		<div class="card p-8 text-center">
			<p class="text-lg text-surface-600-300-token mb-4">No PSETs available</p>
			<button class="btn variant-filled-primary" on:click={openCreateModal}>
				Create Your First PSET
			</button>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
			{#each psets as pset (pset.id)}
				<div class="card p-6">
					<div class="flex justify-between items-start mb-4">
						<div>
							<h3 class="h3 mb-1">{pset.name}</h3>
							<span class="badge variant-filled-surface">ID: {pset.id}</span>
						</div>
					</div>

					{#if pset.description}
						<p class="text-sm text-surface-600-300-token mb-4">{pset.description}</p>
					{/if}

					<div class="space-y-2 mb-4">
						<div class="flex justify-between">
							<span class="font-semibold">Torque Range:</span>
							<span>{pset.torque_min} - {pset.torque_max} Nm</span>
						</div>
						<div class="flex justify-between">
							<span class="font-semibold">Angle Range:</span>
							<span>{pset.angle_min}° - {pset.angle_max}°</span>
						</div>
						<div class="flex justify-between text-sm text-surface-600-300-token">
							<span>Target:</span>
							<span
								>{((pset.torque_min + pset.torque_max) / 2).toFixed(1)} Nm, {((pset.angle_min +
									pset.angle_max) /
									2).toFixed(1)}°</span
							>
						</div>
					</div>

					<div class="flex gap-2">
						<button
							class="btn btn-sm variant-filled-secondary flex-1"
							on:click={() => openEditModal(pset)}
						>
							Edit
						</button>

						{#if deleteConfirmId === pset.id}
							<button
								class="btn btn-sm variant-filled-error flex-1"
								on:click={() => handleDelete(pset.id)}
							>
								Confirm Delete?
							</button>
							<button
								class="btn btn-sm variant-ghost-surface"
								on:click={() => (deleteConfirmId = null)}
							>
								Cancel
							</button>
						{:else}
							<button
								class="btn btn-sm variant-filled-error flex-1"
								on:click={() => (deleteConfirmId = pset.id)}
							>
								Delete
							</button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Modal for Create/Edit -->
{#if showModal}
	<div class="modal-backdrop" on:click={() => (showModal = false)} transition:fade={{ duration: 150 }}>
		<div class="modal-content card p-6" on:click|stopPropagation transition:scale={{ duration: 200, start: 0.95 }}>
			<h2 class="h2 mb-4">{modalMode === 'create' ? 'Create New PSET' : 'Edit PSET'}</h2>

			<form on:submit|preventDefault={handleSubmit} class="space-y-4">
				<label class="label">
					<span>Name *</span>
					<input
						type="text"
						class="input"
						class:input-error={formErrors.name}
						bind:value={formData.name}
						placeholder="e.g., Medium Duty"
					/>
					{#if formErrors.name}
						<p class="text-error-500 text-sm mt-1">{formErrors.name}</p>
					{/if}
				</label>

				<div class="grid grid-cols-2 gap-4">
					<label class="label">
						<span>Torque Min (Nm) *</span>
						<input
							type="number"
							class="input"
							class:input-error={formErrors.torque}
							bind:value={formData.torque_min}
							step="0.1"
							min="0"
						/>
					</label>

					<label class="label">
						<span>Torque Max (Nm) *</span>
						<input
							type="number"
							class="input"
							class:input-error={formErrors.torque}
							bind:value={formData.torque_max}
							step="0.1"
							min="0"
						/>
					</label>
				</div>
				{#if formErrors.torque}
					<p class="text-error-500 text-sm">{formErrors.torque}</p>
				{/if}

				<div class="grid grid-cols-2 gap-4">
					<label class="label">
						<span>Angle Min (°) *</span>
						<input
							type="number"
							class="input"
							class:input-error={formErrors.angle}
							bind:value={formData.angle_min}
							step="0.1"
							min="0"
							max="360"
						/>
					</label>

					<label class="label">
						<span>Angle Max (°) *</span>
						<input
							type="number"
							class="input"
							class:input-error={formErrors.angle}
							bind:value={formData.angle_max}
							step="0.1"
							min="0"
							max="360"
						/>
					</label>
				</div>
				{#if formErrors.angle}
					<p class="text-error-500 text-sm">{formErrors.angle}</p>
				{/if}

				<label class="label">
					<span>Description</span>
					<textarea
						class="textarea"
						bind:value={formData.description}
						rows="3"
						placeholder="Optional description..."
					></textarea>
				</label>

				<div class="bg-surface-100-800-token p-3 rounded">
					<p class="text-sm font-semibold mb-1">Preview:</p>
					<p class="text-sm">
						Target Torque: {((formData.torque_min + formData.torque_max) / 2).toFixed(1)} Nm
					</p>
					<p class="text-sm">
						Target Angle: {((formData.angle_min + formData.angle_max) / 2).toFixed(1)}°
					</p>
				</div>

				<div class="flex gap-2 pt-4">
					<button type="submit" class="btn variant-filled-primary flex-1">
						{modalMode === 'create' ? 'Create' : 'Update'}
					</button>
					<button
						type="button"
						class="btn variant-ghost-surface flex-1"
						on:click={() => (showModal = false)}
					>
						Cancel
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<style>
	.container {
		max-width: 1400px;
	}

	.input-error {
		border-color: rgb(var(--color-error-500)) !important;
	}

	/* Modal styling - Light mode */
	.modal-backdrop {
		position: fixed;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		background: rgba(0, 0, 0, 0.6);
		backdrop-filter: blur(4px);
		display: flex;
		justify-content: center;
		align-items: center;
		z-index: 1000;
	}

	.modal-content {
		max-width: 600px;
		width: 90%;
		max-height: 90vh;
		overflow-y: auto;
		background: linear-gradient(135deg, #ffffff 0%, #fefdfb 100%);
		border: 2px solid #f0e6d2;
		box-shadow: 0 8px 32px rgba(237, 125, 49, 0.15);
	}

	.modal-content:hover {
		box-shadow: 0 8px 32px rgba(237, 125, 49, 0.15);
	}

	/* Modal styling - Dark mode */
	:global(.dark) .modal-backdrop {
		background: rgba(0, 0, 0, 0.8);
	}

	:global(.dark) .modal-content {
		background: linear-gradient(135deg, #1d3832 0%, #18302a 100%);
		border: 2px solid #253d35;
		box-shadow: 0 8px 32px rgba(25, 45, 38, 0.4), 0 0 48px rgba(42, 82, 72, 0.25);
	}

	:global(.dark) .modal-content:hover {
		box-shadow: 0 8px 32px rgba(25, 45, 38, 0.4), 0 0 48px rgba(42, 82, 72, 0.25);
	}
</style>
