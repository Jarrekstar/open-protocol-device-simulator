<script lang="ts">
	import { Button, FormField } from '$lib/components/ui';
	import type { Job, JobStep, Pset } from '$lib/types';

	interface Props {
		mode: 'create' | 'edit';
		initialData?: Job;
		psets: Pset[];
		channelId: number;
		onsubmit: (job: Job) => Promise<void>;
		oncancel: () => void;
	}

	let { mode, initialData, psets, channelId, onsubmit, oncancel }: Props = $props();

	const defaultStep = (): JobStep => ({
		channel_id: channelId,
		pset_id: psets[0]?.id ?? 1,
		auto_value: true,
		batch_size: 1
	});

	function cloneJob(job: Job): Job {
		return {
			...job,
			steps: job.steps.map((step) => ({ ...step }))
		};
	}

	let formData = $state<Job>(
		initialData
			? cloneJob(initialData)
			: {
					id: 0,
					name: '',
					forced_order: 1,
					first_tightening_timeout: 0,
					job_timeout: 0,
					batch_count_mode: 0,
					lock_at_job_done: false,
					use_line_control: false,
					repeat_job: false,
					loosening_mode: 0,
					repair_mode: 0,
					steps: [defaultStep()]
				}
	);
	let errors = $state<Record<string, string>>({});
	let isSubmitting = $state(false);
	let showAdvanced = $state(false);

	const totalTightenings = $derived(
		formData.steps.reduce((total, step) => total + Number(step.batch_size || 0), 0)
	);

	function addStep() {
		if (formData.steps.length < 50) {
			formData.steps.push(defaultStep());
		}
	}

	function removeStep(index: number) {
		if (formData.steps.length > 1) {
			formData.steps.splice(index, 1);
		}
	}

	function moveStep(index: number, offset: number) {
		const target = index + offset;
		if (target < 0 || target >= formData.steps.length) return;
		const [step] = formData.steps.splice(index, 1);
		formData.steps.splice(target, 0, step);
	}

	function validate(): Record<string, string> {
		const next: Record<string, string> = {};
		const nameBytes = new TextEncoder().encode(formData.name).length;
		if (!Number.isInteger(Number(formData.id)) || formData.id < 0 || formData.id > 9999) {
			next.id = 'Job ID must be an integer from 0000 to 9999.';
		}
		if (!formData.name.trim()) next.name = 'Name is required.';
		else if (!/^[\x20-\x7E]*$/.test(formData.name) || nameBytes > 25) {
			next.name = 'Name must use printable ASCII and be no longer than 25 bytes.';
		}
		if (![0, 1, 2].includes(Number(formData.forced_order))) {
			next.forced_order = 'Forced order must be 0, 1, or 2.';
		}
		if (![0, 1].includes(Number(formData.batch_count_mode))) {
			next.batch_count_mode = 'Batch count mode must be 0 or 1.';
		}
		if (
			!Number.isInteger(Number(formData.first_tightening_timeout)) ||
			formData.first_tightening_timeout < 0 ||
			formData.first_tightening_timeout > 9999
		) {
			next.first_tightening_timeout = 'First tightening timeout must be from 0 to 9999.';
		}
		if (
			!Number.isInteger(Number(formData.job_timeout)) ||
			formData.job_timeout < 0 ||
			formData.job_timeout > 99999
		) {
			next.job_timeout = 'Job timeout must be from 0 to 99999.';
		}
		if (![0, 1, 2].includes(Number(formData.loosening_mode))) {
			next.loosening_mode = 'Loosening mode must be 0, 1, or 2.';
		}
		if (![0, 1].includes(Number(formData.repair_mode))) {
			next.repair_mode = 'Repair mode must be 0 or 1.';
		}
		if (formData.steps.length < 1 || formData.steps.length > 50) {
			next.steps = 'A Job must contain between 1 and 50 steps.';
		}
		formData.steps.forEach((step, index) => {
			if (
				!Number.isInteger(Number(step.channel_id)) ||
				step.channel_id < 0 ||
				step.channel_id > 99 ||
				Number(step.channel_id) !== channelId
			) {
				next[`step-${index}`] = `Channel must match configured channel ${channelId}.`;
			} else if (
				!Number.isInteger(Number(step.pset_id)) ||
				!psets.some((pset) => pset.id === Number(step.pset_id))
			) {
				next[`step-${index}`] = 'Select an existing PSET.';
			} else if (
				!Number.isInteger(Number(step.batch_size)) ||
				step.batch_size < 1 ||
				step.batch_size > 99
			) {
				next[`step-${index}`] = 'Batch size must be from 1 to 99.';
			}
		});
		return next;
	}

	async function handleSubmit() {
		errors = validate();
		if (Object.keys(errors).length > 0) return;

		const normalized: Job = {
			...formData,
			id: Number(formData.id),
			forced_order: Number(formData.forced_order),
			first_tightening_timeout: Number(formData.first_tightening_timeout),
			job_timeout: Number(formData.job_timeout),
			batch_count_mode: Number(formData.batch_count_mode),
			loosening_mode: Number(formData.loosening_mode),
			repair_mode: Number(formData.repair_mode),
			steps: formData.steps.map((step) => ({
				channel_id: Number(step.channel_id),
				pset_id: Number(step.pset_id),
				auto_value: Boolean(step.auto_value),
				batch_size: Number(step.batch_size)
			}))
		};

		isSubmitting = true;
		try {
			await onsubmit(normalized);
		} finally {
			isSubmitting = false;
		}
	}
</script>

<form
	class="space-y-6 max-h-[75vh] overflow-y-auto pr-2"
	onsubmit={(event) => {
		event.preventDefault();
		handleSubmit();
	}}
>
	<div class="grid gap-4 md:grid-cols-2">
		<FormField
			label="Job ID"
			type="number"
			bind:value={formData.id}
			min={0}
			max={9999}
			disabled={mode === 'edit'}
			error={errors.id}
			help="Revision 1 uses IDs 00-99."
			required
		/>
		<FormField
			label="Name"
			type="text"
			bind:value={formData.name}
			error={errors.name}
			help="Printable ASCII, maximum 25 bytes."
			required
		/>
		<FormField
			label="Batch Count Mode"
			type="select"
			bind:value={formData.batch_count_mode}
			options={[
				{ value: 0, label: 'Count OK only' },
				{ value: 1, label: 'Count OK and NOK' }
			]}
			error={errors.batch_count_mode}
		/>
		<FormField
			label="Forced Order"
			type="select"
			bind:value={formData.forced_order}
			options={[
				{ value: 0, label: 'Free order' },
				{ value: 1, label: 'Forced order' },
				{ value: 2, label: 'Free order, forced first' }
			]}
			error={errors.forced_order}
			help="Stored for protocol compatibility; execution is sequential in revision 1."
		/>
	</div>

	<div class="rounded-xl border border-surface-200-700-token p-4 space-y-4">
		<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
			<div>
				<h3 class="font-semibold">Ordered Steps</h3>
				<p class="text-sm opacity-70">
					{formData.steps.length} step{formData.steps.length === 1 ? '' : 's'} ·
					{totalTightenings} total tightenings
				</p>
			</div>
			<Button type="button" onclick={addStep} disabled={formData.steps.length >= 50}>
				Add Step
			</Button>
		</div>

		{#if errors.steps}
			<p class="text-sm text-error-500">{errors.steps}</p>
		{/if}

		<div class="space-y-3">
			{#each formData.steps as step, index}
				<div class="rounded-lg bg-surface-100-800-token p-4">
					<div class="mb-3 flex items-center justify-between">
						<span class="font-semibold">Step {index + 1}</span>
						<div class="flex gap-2">
							<button
								type="button"
								class="btn btn-sm variant-ghost-surface"
								onclick={() => moveStep(index, -1)}
								disabled={index === 0}
								aria-label="Move step up"
							>Up</button>
							<button
								type="button"
								class="btn btn-sm variant-ghost-surface"
								onclick={() => moveStep(index, 1)}
								disabled={index === formData.steps.length - 1}
								aria-label="Move step down"
							>Down</button>
							<button
								type="button"
								class="btn btn-sm variant-ghost-surface text-error-500"
								onclick={() => removeStep(index)}
								disabled={formData.steps.length === 1}
							>Remove</button>
						</div>
					</div>
					<div class="grid gap-4 md:grid-cols-4">
						<label class="label">
							<span>PSET</span>
							<select class="select" bind:value={step.pset_id}>
								{#each psets as pset}
									<option value={pset.id}>{pset.name} ({pset.id})</option>
								{/each}
							</select>
						</label>
						<FormField
							label="Channel"
							type="number"
							bind:value={step.channel_id}
							min={0}
							max={99}
						/>
						<FormField
							label="Batch Size"
							type="number"
							bind:value={step.batch_size}
							min={1}
							max={99}
						/>
						<FormField
							label="Auto Value"
							type="checkbox"
							bind:value={step.auto_value}
							help="Informational in the simulator."
						/>
					</div>
					{#if errors[`step-${index}`]}
						<p class="mt-2 text-sm text-error-500">{errors[`step-${index}`]}</p>
					{/if}
				</div>
			{/each}
		</div>
	</div>

	<div class="rounded-xl border border-surface-200-700-token p-4">
		<button
			type="button"
			class="flex w-full items-center justify-between font-semibold"
			onclick={() => (showAdvanced = !showAdvanced)}
		>
			<span>Advanced Settings</span>
			<span>{showAdvanced ? 'Hide' : 'Show'}</span>
		</button>
		{#if showAdvanced}
			<div class="mt-4 grid gap-4 md:grid-cols-2">
				<FormField
					label="First Tightening Timeout"
					type="number"
					bind:value={formData.first_tightening_timeout}
					min={0}
					max={9999}
					error={errors.first_tightening_timeout}
					help="Informational, 0000-9999."
				/>
				<FormField
					label="Job Timeout"
					type="number"
					bind:value={formData.job_timeout}
					min={0}
					max={99999}
					error={errors.job_timeout}
					help="Informational, 00000-99999."
				/>
				<FormField
					label="Loosening Mode"
					type="select"
					bind:value={formData.loosening_mode}
					options={[
						{ value: 0, label: 'Disabled' },
						{ value: 1, label: 'Enabled' },
						{ value: 2, label: 'Enabled with confirmation' }
					]}
					error={errors.loosening_mode}
					help="Informational."
				/>
				<FormField
					label="Repair / Reserved Mode"
					type="select"
					bind:value={formData.repair_mode}
					options={[
						{ value: 0, label: 'Disabled' },
						{ value: 1, label: 'Enabled' }
					]}
					error={errors.repair_mode}
					help="Informational."
				/>
				<FormField label="Lock tool at job done" type="checkbox" bind:value={formData.lock_at_job_done} />
				<FormField label="Repeat job" type="checkbox" bind:value={formData.repeat_job} />
				<FormField
					label="Use line control"
					type="checkbox"
					bind:value={formData.use_line_control}
					help="Informational."
				/>
			</div>
		{/if}
	</div>

	<div class="flex justify-end gap-3">
		<Button type="button" variant="ghost-surface" onclick={oncancel} disabled={isSubmitting}>
			Cancel
		</Button>
		<Button type="submit" disabled={isSubmitting || psets.length === 0}>
			{isSubmitting ? 'Saving...' : mode === 'create' ? 'Create Job' : 'Update Job'}
		</Button>
	</div>
</form>
