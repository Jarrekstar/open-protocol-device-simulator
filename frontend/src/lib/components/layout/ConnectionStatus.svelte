<script lang="ts">
	import { connected } from '$lib/stores/websocket';
	import StatusIndicator from '$lib/components/ui/StatusIndicator.svelte';
</script>

<StatusIndicator
	status={$connected ? 'connected' : 'disconnected'}
	size="sm"
	pulse={$connected}
/>
