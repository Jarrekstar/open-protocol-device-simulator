<script lang="ts">
	interface Props {
		enabled: boolean;
	}

	let { enabled }: Props = $props();
</script>

<div class="text-sm text-surface-600-300-token">
	Tool {enabled ? 'enabled' : 'disabled'}
</div>
