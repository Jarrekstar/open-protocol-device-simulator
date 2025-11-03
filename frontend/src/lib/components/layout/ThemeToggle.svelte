<script lang="ts">
	import { theme, toggleTheme } from '$lib/stores/theme';
</script>

<button class="btn btn-sm variant-ghost-surface" onclick={toggleTheme} title="Toggle theme">
	{#if $theme === 'light'}
		🌙
	{:else}
		☀️
	{/if}
</button>
