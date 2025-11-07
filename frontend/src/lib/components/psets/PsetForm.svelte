<script lang="ts">
	import { FormField, Button, RangeSlider } from '$lib/components/ui';
	import { validatePset } from '$lib/utils';
	import type { Pset } from '$lib/types';

	interface Props {
		mode: 'create' | 'edit';
		initialData?: Partial<Pset>;
		onsubmit: (data: Partial<Pset>) => Promise<void>;
		oncancel: () => void;
	}

	let { mode, initialData, onsubmit, oncancel }: Props = $props();

	let formData = $state({
		name: initialData?.name || '',
		torque_min: initialData?.torque_min ?? 10.0,
		torque_max: initialData?.torque_max ?? 15.0,
		angle_min: initialData?.angle_min ?? 30.0,
		angle_max: initialData?.angle_max ?? 45.0,
		description: initialData?.description || ''
	});

	let formErrors = $state<Record<string, string>>({});
	let isSubmitting = $state(false);

	async function handleSubmit() {
		// Validate form
		formErrors = validatePset(formData);

		if (Object.keys(formErrors).length > 0) {
			return;
		}

		isSubmitting = true;
		try {
			await onsubmit(formData);
		} finally {
			isSubmitting = false;
		}
	}
</script>

<form
	onsubmit={(e) => {
		e.preventDefault();
		handleSubmit();
	}}
	class="space-y-6"
>
	<FormField
		label="Name"
		type="text"
		bind:value={formData.name}
		placeholder="e.g., Medium Duty"
		required={true}
		error={formErrors.name}
		class={formErrors.name ? 'input-error' : ''}
	/>

	<div>
		<RangeSlider
			label="Torque Range"
			min={0}
			max={200}
			bind:valueMin={formData.torque_min}
			bind:valueMax={formData.torque_max}
			step={0.1}
			unit="Nm"
		/>
		{#if formErrors.torque}
			<p class="text-error-500 text-sm mt-1">{formErrors.torque}</p>
		{/if}
		<div class="grid grid-cols-2 gap-4 mt-3">
			<FormField
				label="Min"
				type="number"
				bind:value={formData.torque_min}
				step={0.1}
				min={0}
				required={true}
				inputClass={formErrors.torque ? 'border-error-500' : ''}
			/>
			<FormField
				label="Max"
				type="number"
				bind:value={formData.torque_max}
				step={0.1}
				min={0}
				required={true}
				inputClass={formErrors.torque ? 'border-error-500' : ''}
			/>
		</div>
	</div>

	<div>
		<RangeSlider
			label="Angle Range"
			min={0}
			max={360}
			bind:valueMin={formData.angle_min}
			bind:valueMax={formData.angle_max}
			step={0.1}
			unit="°"
		/>
		{#if formErrors.angle}
			<p class="text-error-500 text-sm mt-1">{formErrors.angle}</p>
		{/if}
		<div class="grid grid-cols-2 gap-4 mt-3">
			<FormField
				label="Min"
				type="number"
				bind:value={formData.angle_min}
				step={0.1}
				min={0}
				max={360}
				required={true}
				inputClass={formErrors.angle ? 'border-error-500' : ''}
			/>
			<FormField
				label="Max"
				type="number"
				bind:value={formData.angle_max}
				step={0.1}
				min={0}
				max={360}
				required={true}
				inputClass={formErrors.angle ? 'border-error-500' : ''}
			/>
		</div>
	</div>

	<FormField
		label="Description (optional)"
		type="textarea"
		bind:value={formData.description}
		placeholder="Optional notes about this PSET"
	/>

	<div class="flex justify-end gap-3 pt-2">
		<Button variant="ghost-surface" type="button" onclick={oncancel} disabled={isSubmitting}>
			Cancel
		</Button>
		<Button type="submit" disabled={isSubmitting}>
			{mode === 'create' ? 'Create PSET' : 'Update PSET'}
		</Button>
	</div>
</form>
