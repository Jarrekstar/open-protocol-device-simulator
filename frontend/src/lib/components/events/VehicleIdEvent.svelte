<script lang="ts">
	interface Props {
		vin: string;
	}

	let { vin }: Props = $props();
</script>

<div class="text-sm text-surface-600-300-token">
	VIN updated to {vin}
</div>
