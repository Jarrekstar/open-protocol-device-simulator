<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/client';
	import { showToast } from '$lib/stores/ui';
	import { formatErrorMessage } from '$lib/utils';
	import { Badge, Button, FormField, Section } from '$lib/components/ui';
	import type {
		MidFamilyDefinition,
		ProtocolProfile,
		RevisionPolicy
	} from '$lib/types';

	let catalog = $state<MidFamilyDefinition[]>([]);
	let profile = $state<ProtocolProfile | null>(null);
	let isLoading = $state(true);
	let isSaving = $state(false);
	let loadError = $state<string | null>(null);

	function formatMid(mid: number) {
		return mid.toString().padStart(4, '0');
	}

	function updateFamily(
		familyId: string,
		field: 'enabled' | 'policy' | 'revision',
		value: boolean | RevisionPolicy | number
	) {
		if (!profile) return;
		profile = {
			...profile,
			families: {
				...profile.families,
				[familyId]: {
					...profile.families[familyId],
					[field]: value
				}
			}
		};
	}

	async function load() {
		isLoading = true;
		loadError = null;
		try {
			[catalog, profile] = await Promise.all([
				api.getProtocolCatalog(),
				api.getProtocolProfile()
			]);
		} catch (error) {
			loadError = formatErrorMessage('load protocol configuration', error);
		} finally {
			isLoading = false;
		}
	}

	async function save() {
		if (!profile) return;
		isSaving = true;
		try {
			await api.validateProtocolProfile(profile);
			profile = await api.updateProtocolProfile(profile);
			showToast({ type: 'success', message: 'Protocol revision profile updated' });
		} catch (error) {
			showToast({
				type: 'error',
				message: formatErrorMessage('update protocol profile', error)
			});
		} finally {
			isSaving = false;
		}
	}

	onMount(load);
</script>

<svelte:head>
	<title>Protocol Revisions - Device Simulator</title>
</svelte:head>

<div class="space-y-6 animate-fade-in">
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-3xl font-semibold text-surface-900 dark:text-surface-900">
				Protocol Revisions
			</h1>
			<p class="mt-1 text-sm text-surface-600 dark:text-surface-800">
				Configure revision acceptance by MID family and edit revision-specific sample data.
			</p>
		</div>
		<Button onclick={save} disabled={isSaving || !profile}>
			{isSaving ? 'Applying...' : 'Apply Profile'}
		</Button>
	</div>

	{#if isLoading}
		<div class="card p-8 text-center text-surface-700 dark:text-surface-800">
			Loading protocol catalog...
		</div>
	{:else if loadError}
		<div class="card space-y-4 border-error-500 p-6 dark:border-error-400/60">
			<p class="text-error-700 dark:text-error-300">{loadError}</p>
			<Button onclick={load}>Retry</Button>
		</div>
	{:else if profile}
		<Section
			title="MID Families"
			description="Exact accepts only the selected revision. Any implemented accepts every completed codec for that family."
		>
			<div
				class="overflow-hidden rounded-xl border border-surface-200 dark:border-surface-700"
			>
				<div class="overflow-x-auto">
					<table class="min-w-[900px] w-full">
						<thead
							class="border-b border-surface-200 bg-surface-100 dark:border-surface-700 dark:bg-surface-300"
						>
							<tr>
								<th
									class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-surface-400 dark:text-surface-300"
									>Family</th
								>
								<th
									class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-surface-400 dark:text-surface-300"
									>MIDs</th
								>
								<th
									class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-surface-400 dark:text-surface-300"
									>Policy</th
								>
								<th
									class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-surface-400 dark:text-surface-300"
									>Revision</th
								>
								<th
									class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-surface-400 dark:text-surface-300"
									>Specification</th
								>
								<th
									class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-surface-400 dark:text-surface-300"
									>Availability</th
								>
							</tr>
						</thead>
						<tbody class="divide-y divide-surface-200 dark:divide-surface-700">
							{#each catalog as family}
								{@const selection = profile.families[family.id]}
								<tr
									class="transition-colors hover:bg-surface-50 dark:hover:bg-surface-800/50"
								>
									<td class="px-4 py-3">
										<div class="font-medium text-surface-900 dark:text-surface-100">
											{family.name}
										</div>
										<div class="mt-0.5 text-xs text-surface-500 dark:text-surface-400">
											{family.id}
										</div>
									</td>
									<td
										class="px-4 py-3 font-mono text-sm text-surface-600 dark:text-surface-400"
									>
										{family.mids.map(formatMid).join(', ')}
									</td>
									<td class="px-4 py-3">
										<select
											class="select min-w-40"
											value={selection.policy}
											onchange={(event) =>
												updateFamily(
													family.id,
													'policy',
													event.currentTarget.value as RevisionPolicy
												)}
										>
											<option value="exact">Exact</option>
											<option value="any_implemented">Any implemented</option>
										</select>
									</td>
									<td class="px-4 py-3">
										<select
											class="select min-w-24"
											value={selection.revision}
											disabled={selection.policy === 'any_implemented'}
											onchange={(event) =>
												updateFamily(family.id, 'revision', Number(event.currentTarget.value))}
										>
											{#each family.implemented_revisions as revision}
												<option value={revision}>{revision}</option>
											{/each}
										</select>
									</td>
									<td class="px-4 py-3 text-sm text-surface-600 dark:text-surface-400">
										{family.supported_revisions.join(', ')}
									</td>
									<td class="px-4 py-3">
										<Badge variant="filled-success">
											{family.implemented_revisions.length} implemented
										</Badge>
										{#if family.implemented_revisions.length < family.supported_revisions.length}
											<div class="mt-1 text-xs text-warning-700 dark:text-warning-300">
												Remaining revisions are visible but not selectable yet.
											</div>
										{/if}
									</td>
								</tr>
								<tr class="bg-surface-50/60 dark:bg-surface-200/60">
									<td colspan="6" class="px-4 py-3">
										<div class="flex flex-wrap gap-2 text-xs">
											{#each family.features as feature}
												<Badge variant="soft">
													Rev {feature.revision}: {feature.summary}
												</Badge>
											{/each}
										</div>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</div>
		</Section>

		<Section
			title="Revision Sample Data"
			description="These values populate fields introduced by later Communication, Vehicle ID, Job, tightening, and multi-spindle revisions."
		>
			<div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
				<FormField
					label="Open Protocol Version"
					bind:value={profile.samples.open_protocol_version}
					help="MID 0002 revision 3, up to 19 ASCII characters."
				/>
				<FormField
					label="Controller Software Version"
					bind:value={profile.samples.controller_software_version}
				/>
				<FormField
					label="Tool Software Version"
					bind:value={profile.samples.tool_software_version}
				/>
				<FormField label="RBU Type" bind:value={profile.samples.rbu_type} />
				<FormField
					label="Controller Serial Number"
					bind:value={profile.samples.controller_serial_number}
				/>
				<FormField
					label="Station ID"
					type="number"
					min={0}
					max={99}
					bind:value={profile.samples.station_id}
				/>
				<FormField
					label="System Type"
					type="number"
					min={0}
					max={1}
					bind:value={profile.samples.system_type}
					help="0 = tightening system, 1 = press system."
				/>
				<FormField
					label="System Subtype"
					type="number"
					min={0}
					max={2}
					bind:value={profile.samples.system_subtype}
				/>
				<FormField
					label="Identifier Part 1"
					bind:value={profile.samples.identifier_part_1}
					help="Example VIN used by MID 0035 revision 5."
				/>
				<FormField
					label="Identifier Part 2"
					bind:value={profile.samples.identifier_part_2}
					help="Example work order for MID 0052 revision 2."
				/>
				<FormField
					label="Identifier Part 3"
					bind:value={profile.samples.identifier_part_3}
					help="Example model or variant identifier."
				/>
				<FormField
					label="Identifier Part 4"
					bind:value={profile.samples.identifier_part_4}
					help="Example body or serial identifier."
				/>
				<FormField
					label="Job Sequence Number"
					type="number"
					min={0}
					max={65535}
					bind:value={profile.samples.job_sequence_number}
					help="MID 0035 revision 5, MID 0061 revision 2+, and MID 0101 revision 5."
				/>
				<FormField
					label="Job Tightening Status"
					type="number"
					min={0}
					max={10}
					bind:value={profile.samples.job_tightening_status}
					help="MID 0035 revision 4+, 00-10."
				/>
				<FormField
					label="Tool Serial Number"
					bind:value={profile.samples.tool_serial_number}
					help="MID 0061 revision 2+, up to 14 ASCII characters."
				/>
			</div>

			<div class="grid gap-3 md:grid-cols-2">
				<label
					class="flex items-center gap-3 rounded-lg border border-surface-200 bg-surface-50/70 p-3 text-surface-800 dark:border-surface-400 dark:bg-surface-200/80 dark:text-surface-800"
				>
					<input
						type="checkbox"
						class="checkbox"
						bind:checked={profile.samples.supports_sequence_number}
					/>
					<span>MID 0002 advertises sequence-number support</span>
				</label>
				<label
					class="flex items-center gap-3 rounded-lg border border-surface-200 bg-surface-50/70 p-3 text-surface-800 dark:border-surface-400 dark:bg-surface-200/80 dark:text-surface-800"
				>
					<input
						type="checkbox"
						class="checkbox"
						bind:checked={profile.samples.supports_message_linking}
					/>
					<span>MID 0002 advertises message-linking support</span>
				</label>
			</div>
		</Section>

		<Section
			title="Tightening Revision Samples"
			description="Torque-like values use protocol scaling: values are stored as hundredths. Angle fields use whole degrees unless marked decimal."
		>
			<div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
				<FormField
					label="Strategy"
					type="number"
					min={0}
					max={99}
					bind:value={profile.samples.tightening_strategy}
				/>
				<FormField
					label="Strategy Options"
					type="number"
					min={0}
					max={99999}
					bind:value={profile.samples.tightening_strategy_options}
				/>
				<FormField
					label="Error Status"
					type="number"
					min={0}
					max={4294967295}
					bind:value={profile.samples.tightening_error_status}
				/>
				<FormField
					label="Error Status 2"
					type="number"
					min={0}
					max={4294967295}
					bind:value={profile.samples.tightening_error_status_2}
				/>
				<FormField
					label="Rundown Angle Min"
					type="number"
					min={0}
					max={99999}
					bind:value={profile.samples.rundown_angle_min}
				/>
				<FormField
					label="Rundown Angle Max"
					type="number"
					min={0}
					max={99999}
					bind:value={profile.samples.rundown_angle_max}
				/>
				<FormField
					label="Rundown Angle"
					type="number"
					min={0}
					max={99999}
					bind:value={profile.samples.rundown_angle}
				/>
				<FormField
					label="Current Min (%)"
					type="number"
					min={0}
					max={999}
					bind:value={profile.samples.current_monitoring_min}
				/>
				<FormField
					label="Current Max (%)"
					type="number"
					min={0}
					max={999}
					bind:value={profile.samples.current_monitoring_max}
				/>
				<FormField
					label="Current Value (%)"
					type="number"
					min={0}
					max={999}
					bind:value={profile.samples.current_monitoring_value}
				/>
				<FormField
					label="Self-tap Min ×100"
					type="number"
					min={0}
					max={999999}
					bind:value={profile.samples.self_tap_min}
				/>
				<FormField
					label="Self-tap Max ×100"
					type="number"
					min={0}
					max={999999}
					bind:value={profile.samples.self_tap_max}
				/>
				<FormField
					label="Self-tap Torque ×100"
					type="number"
					min={0}
					max={999999}
					bind:value={profile.samples.self_tap_torque}
				/>
				<FormField
					label="Prevail Min ×100"
					type="number"
					min={0}
					max={999999}
					bind:value={profile.samples.prevail_torque_min}
				/>
				<FormField
					label="Prevail Max ×100"
					type="number"
					min={0}
					max={999999}
					bind:value={profile.samples.prevail_torque_max}
				/>
				<FormField
					label="Prevail Torque ×100"
					type="number"
					min={0}
					max={999999}
					bind:value={profile.samples.prevail_torque}
				/>
				<FormField
					label="PVT Compensation ×100"
					type="number"
					min={0}
					max={999999}
					bind:value={profile.samples.prevail_torque_compensate}
				/>
				<FormField
					label="Torque Unit"
					type="number"
					min={1}
					max={8}
					bind:value={profile.samples.torque_unit}
					help="1 = Nm."
				/>
				<FormField
					label="Result Type"
					type="number"
					min={1}
					max={8}
					bind:value={profile.samples.tightening_result_type}
					help="1 = tightening."
				/>
				<FormField
					label="Customer Error Code"
					bind:value={profile.samples.customer_tightening_error_code}
					help="MID 0061 revision 5+, four ASCII characters."
				/>
				<FormField
					label="Compensated Angle ×100"
					type="number"
					min={0}
					max={9999999}
					bind:value={profile.samples.compensated_angle}
				/>
				<FormField
					label="Final Angle Decimal ×100"
					type="number"
					min={0}
					max={9999999}
					bind:value={profile.samples.final_angle_decimal}
				/>
				<FormField
					label="Multistage Count"
					type="number"
					min={1}
					max={99}
					bind:value={profile.samples.multistage_count}
				/>
				<FormField
					label="Stage Torque ×100"
					type="number"
					min={0}
					max={999999}
					bind:value={profile.samples.multistage_torque}
				/>
				<FormField
					label="Stage Angle"
					type="number"
					min={0}
					max={99999}
					bind:value={profile.samples.multistage_angle}
				/>
			</div>
		</Section>

		<Section
			title="Multi-spindle Subscription Sample"
			description="Example MID 0100 revision 2-5 request values used when testing subscription payloads."
		>
			<div class="grid gap-4 md:grid-cols-2">
				<FormField
					label="Data Number"
					type="number"
					min={0}
					max={9999999999}
					bind:value={profile.samples.multi_spindle_data_number}
				/>
				<label
					class="flex items-center gap-3 rounded-lg border border-surface-200 bg-surface-50/70 p-3 text-surface-800 dark:border-surface-400 dark:bg-surface-200/80 dark:text-surface-800"
				>
					<input
						type="checkbox"
						class="checkbox"
						bind:checked={profile.samples.multi_spindle_send_only_new}
					/>
					<span>Send only new result data in revision 3-5 subscription samples</span>
				</label>
			</div>
		</Section>
	{/if}
</div>
