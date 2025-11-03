<script lang="ts">
	interface Props {
		psetId: number;
		psetName: string;
	}

	let { psetId, psetName }: Props = $props();
</script>

<div class="text-sm text-surface-600-300-token">
	PSET changed to {psetName} (ID {psetId})
</div>
