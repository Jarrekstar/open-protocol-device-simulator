<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/client';
	import { JobCard, JobModal, JobRuntimePanel } from '$lib/components/jobs';
	import { Button, EmptyState } from '$lib/components/ui';
	import { deviceState } from '$lib/stores/device';
	import { showToast } from '$lib/stores/ui';
	import { formatErrorMessage } from '$lib/utils';
	import { refreshDeviceState } from '$lib/stores/websocket';
	import type { Job, Pset } from '$lib/types';

	let jobs: Job[] = $state([]);
	let psets: Pset[] = $state([]);
	let loading = $state(true);
	let searchQuery = $state('');
	let modalOpen = $state(false);
	let modalMode: 'create' | 'edit' = $state('create');
	let editingJob: Job | null = $state(null);

	const running = $derived($deviceState?.current_job_status === 'running');
	const jobModeSelected = $derived($deviceState?.operation_mode === 'job');
	const filteredJobs = $derived.by(() => {
		const query = searchQuery.trim().toLowerCase();
		if (!query) return jobs;
		return jobs.filter(
			(job) =>
				job.name.toLowerCase().includes(query) ||
				job.id.toString().padStart(2, '0').includes(query) ||
				job.steps.some((step) => step.pset_id.toString().includes(query))
		);
	});

	async function loadData() {
		loading = true;
		try {
			[jobs, psets] = await Promise.all([api.getJobs(), api.getPsets()]);
		} catch (error) {
			showToast({ type: 'error', message: formatErrorMessage('load Jobs', error) });
		} finally {
			loading = false;
		}
	}

	function openCreate() {
		modalMode = 'create';
		editingJob = null;
		modalOpen = true;
	}

	function openEdit(job: Job) {
		modalMode = 'edit';
		editingJob = job;
		modalOpen = true;
	}

	async function saveJob(job: Job) {
		try {
			if (modalMode === 'create') {
				await api.createJob(job);
				showToast({ type: 'success', message: `Job ${job.id.toString().padStart(2, '0')} created` });
			} else {
				await api.updateJob(job.id, job);
				showToast({ type: 'success', message: `Job ${job.id.toString().padStart(2, '0')} updated` });
			}
			modalOpen = false;
			await loadData();
		} catch (error) {
			showToast({ type: 'error', message: formatErrorMessage('save Job', error) });
			throw error;
		}
	}

	async function deleteJob(id: number) {
		if (!window.confirm(`Delete Job ${id.toString().padStart(2, '0')}?`)) return;
		try {
			await api.deleteJob(id);
			showToast({ type: 'success', message: 'Job deleted' });
			await loadData();
		} catch (error) {
			showToast({ type: 'error', message: formatErrorMessage('delete Job', error) });
		}
	}

	async function selectJob(id: number) {
		if (!jobModeSelected) return;
		try {
			await api.selectJob(id);
			await refreshDeviceState();
			showToast({ type: 'success', message: `Job ${id.toString().padStart(2, '0')} selected` });
		} catch (error) {
			showToast({ type: 'error', message: formatErrorMessage('select Job', error) });
		}
	}

	async function restartJob(id: number) {
		try {
			await api.restartJob(id);
			await refreshDeviceState();
			showToast({ type: 'success', message: `Job ${id.toString().padStart(2, '0')} restarted` });
		} catch (error) {
			showToast({ type: 'error', message: formatErrorMessage('restart Job', error) });
		}
	}

	async function clearJob() {
		try {
			await api.clearActiveJob();
			await refreshDeviceState();
			showToast({ type: 'success', message: 'JobMode exited' });
		} catch (error) {
			showToast({ type: 'error', message: formatErrorMessage('exit JobMode', error) });
		}
	}

	onMount(loadData);
</script>

<svelte:head>
	<title>Jobs - Device Simulator</title>
</svelte:head>

<div class="space-y-6 animate-fade-in">
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-3xl font-semibold">Jobs</h1>
			<p class="mt-1 text-sm opacity-70">
				Configure persistent revision 1 Jobs and control JobMode execution.
			</p>
		</div>
		<Button onclick={openCreate} disabled={psets.length === 0}>Create Job</Button>
	</div>

	<JobRuntimePanel onRestart={restartJob} onClear={clearJob} />

	{#if !jobModeSelected}
		<p class="rounded-md bg-warning-50 p-3 text-sm text-warning-700 dark:bg-warning-900/20">
			Select Job mode on the Control page before starting a Job.
		</p>
	{/if}

	{#if jobs.length > 0}
		<input
			class="input max-w-md"
			type="search"
			placeholder="Search by name, Job ID, or PSET..."
			bind:value={searchQuery}
		/>
	{/if}

	{#if loading}
		<div class="card p-8 text-center">Loading Jobs...</div>
	{:else if psets.length === 0}
		<EmptyState
			title="Create a PSET First"
			description="Every Job step must reference an existing PSET."
			icon="!"
		>
			{#snippet action()}
				<a class="btn variant-filled-primary" href="/psets">Open PSETs</a>
			{/snippet}
		</EmptyState>
	{:else if jobs.length === 0}
		<EmptyState
			title="No Jobs Configured"
			description="Jobs are created through this page or the REST API and exposed by Open Protocol MID 0030-0033."
			icon="J"
		>
			{#snippet action()}
				<Button onclick={openCreate}>Create First Job</Button>
			{/snippet}
		</EmptyState>
	{:else if filteredJobs.length === 0}
		<EmptyState title="No Matching Jobs" description="Change or clear the search query." icon="?" />
	{:else}
		<div class="grid auto-rows-fr gap-5 md:grid-cols-2 xl:grid-cols-3">
			{#each filteredJobs as job (job.id)}
				<JobCard
					{job}
					active={$deviceState?.current_job_status != null &&
						$deviceState?.current_job_id === job.id}
					{running}
					selectionEnabled={jobModeSelected}
					onEdit={openEdit}
					onDelete={deleteJob}
					onSelect={selectJob}
				/>
			{/each}
		</div>
	{/if}
</div>

<JobModal
	open={modalOpen}
	mode={modalMode}
	job={editingJob}
	{psets}
	channelId={$deviceState?.channel_id ?? 1}
	onsubmit={saveJob}
	onclose={() => (modalOpen = false)}
/>
