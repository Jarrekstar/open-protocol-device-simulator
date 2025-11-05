<script lang="ts">
	import { connected } from '$lib/stores/websocket';
</script>

<div class="flex items-center gap-2" role="status" aria-live="polite">
	<div
		class="w-2 h-2 rounded-full"
		class:bg-success-500={$connected}
		class:bg-error-500={!$connected}
	></div>
	<span class="text-sm">{$connected ? 'Connected' : 'Disconnected'}</span>
</div>
