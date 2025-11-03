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

<div class="mx-auto max-w-6xl space-y-6">
	<header class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="h1">Parameter Sets (PSETs)</h1>
			<p class="text-sm opacity-70">Manage reusable torque and angle presets for quick selection.</p>
		</div>
		<button class="btn variant-filled-primary sm:w-auto" on:click={openCreateModal}>
			<span>+ Create New PSET</span>
		</button>
	</header>

	{#if loading}
		<div class="flex h-64 items-center justify-center">
			<div class="text-lg text-surface-500">Loading PSETs...</div>
		</div>
	{:else if psets.length === 0}
		<div class="card p-10 text-center">
			<p class="text-lg text-surface-600-300-token">No PSETs available yet.</p>
			<p class="mt-2 text-sm opacity-70">Create one to define tightening targets for your scenarios.</p>
			<button class="btn variant-filled-primary mt-4" on:click={openCreateModal}>
				Create Your First PSET
			</button>
		</div>
	{:else}
		<div class="grid auto-rows-fr grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
			{#each psets as pset (pset.id)}
				<article class="card flex h-full flex-col gap-4 p-6">
					<header class="rounded-lg bg-surface-100-800-token p-4">
						<h3 class="text-lg font-semibold">{pset.name}</h3>
						<p class="text-sm opacity-60">ID: {pset.id}</p>
					</header>

					{#if pset.description}
						<p class="text-sm text-surface-600-300-token">{pset.description}</p>
					{:else}
						<p class="text-sm opacity-70">No description provided.</p>
					{/if}

					<dl class="grid grid-cols-2 gap-4 rounded-lg border border-surface-200-700-token bg-surface-100-800-token p-4 text-sm">
						<div>
							<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Torque Range</dt>
							<dd class="mt-1 font-semibold text-surface-600-300-token">
								{pset.torque_min} – {pset.torque_max} Nm
							</dd>
						</div>
						<div>
							<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Angle Range</dt>
							<dd class="mt-1 font-semibold text-surface-600-300-token">
								{pset.angle_min}° – {pset.angle_max}°
							</dd>
						</div>
						<div>
							<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Target Torque</dt>
							<dd class="mt-1 font-semibold text-surface-600-300-token">
								{((pset.torque_min + pset.torque_max) / 2).toFixed(1)} Nm
							</dd>
						</div>
						<div>
							<dt class="text-xs uppercase tracking-wide text-surface-600-300-token">Target Angle</dt>
							<dd class="mt-1 font-semibold text-surface-600-300-token">
								{((pset.angle_min + pset.angle_max) / 2).toFixed(1)}°
							</dd>
						</div>
					</dl>

					<footer class="mt-auto space-y-3 border-t border-surface-200-700-token pt-4">
						<div class="flex gap-2">
							<button
								class="btn btn-sm variant-filled-secondary flex-1"
								on:click={() => openEditModal(pset)}
							>
								Edit
							</button>
							<button
								class="btn btn-sm variant-ghost-surface btn-error flex-1"
								on:click={() => (deleteConfirmId = deleteConfirmId === pset.id ? null : pset.id)}
							>
								Delete
							</button>
						</div>

						{#if deleteConfirmId === pset.id}
							<div class="space-y-3 rounded-lg border border-error-500 bg-surface-100-800-token p-3 text-sm text-error-600">
								<p>Delete {pset.name}? This action cannot be undone.</p>
								<div class="flex gap-2">
									<button
										class="btn btn-sm variant-filled-error flex-1"
										on:click={() => handleDelete(pset.id)}
									>
										Confirm
									</button>
									<button
										class="btn btn-sm variant-ghost-surface flex-1"
										on:click={() => (deleteConfirmId = null)}
									>
										Cancel
									</button>
								</div>
							</div>
						{/if}
					</footer>
				</article>
			{/each}
		</div>
	{/if}
</div>

<!-- Modal for Create/Edit -->
{#if showModal}
	<div class="modal-backdrop" on:click={() => (showModal = false)} transition:fade={{ duration: 150 }}>
		<div
			class="modal-content rounded-2xl border border-surface-200-700-token bg-surface-100-800-token p-6 shadow-2xl"
			on:click|stopPropagation
			transition:scale={{ duration: 200, start: 0.95 }}
		>
			<header class="mb-4">
				<h2 class="h2">{modalMode === 'create' ? 'Create New PSET' : 'Edit PSET'}</h2>
				<p class="text-sm opacity-70">
					Define the torque and angle window that devices should target.
				</p>
			</header>

			<form on:submit|preventDefault={handleSubmit} class="space-y-5">
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
					<button type="submit" class="btn variant-filled-primary flex-1 sm:flex-none sm:px-8">
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
	.input-error {
		border-color: rgb(var(--color-error-500)) !important;
	}

	.modal-backdrop {
		position: fixed;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		background: rgb(var(--color-surface-900) / 0.6);
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
	}

	:global(.dark) .modal-backdrop {
		background: rgb(var(--color-surface-50) / 0.8);
	}

</style>
