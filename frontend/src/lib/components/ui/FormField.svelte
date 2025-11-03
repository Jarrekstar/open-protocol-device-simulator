<script lang="ts">
	/**
	 * FormField component for consistent form styling
	 */
	interface Props {
		/** Label text */
		label: string;
		/** Input type */
		type?: 'text' | 'number' | 'select' | 'checkbox' | 'textarea';
		/** Value binding */
		value: any;
		/** Additional CSS classes */
		class?: string;
		/** Input-specific props */
		min?: number;
		max?: number;
		step?: number | string;
		placeholder?: string;
		disabled?: boolean;
		/** For select inputs */
		options?: Array<{ value: any; label: string }>;
		/** Helper text */
		help?: string;
	}

	let {
		label,
		type = 'text',
		value = $bindable(),
		class: className = '',
		min,
		max,
		step,
		placeholder,
		disabled = false,
		options = [],
		help
	}: Props = $props();
</script>

<label class="label {className}">
	<span class="text-surface-600-300-token">{label}</span>

	{#if type === 'select'}
		<select class="select" bind:value {disabled}>
			{#each options as option}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
	{:else if type === 'textarea'}
		<textarea
			class="textarea"
			bind:value
			{placeholder}
			{disabled}
		></textarea>
	{:else if type === 'checkbox'}
		<input
			type="checkbox"
			class="checkbox"
			bind:checked={value}
			{disabled}
		/>
	{:else}
		<input
			{type}
			class="input"
			bind:value
			{min}
			{max}
			{step}
			{placeholder}
			{disabled}
		/>
	{/if}

	{#if help}
		<p class="text-xs opacity-70 mt-1">{help}</p>
	{/if}
</label>
