<script lang="ts">
	interface Props {
		total: number;
	}

	let { total }: Props = $props();
</script>

<div class="text-sm text-surface-600-300-token">
	Batch completed · Total {total}
</div>
