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
		required?: boolean;
		/** For select inputs */
		options?: Array<{ value: any; label: string }>;
		/** Helper text */
		help?: string;
		/** Error message */
		error?: string;
		/** Input class override */
		inputClass?: string;
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
		required = false,
		options = [],
		help,
		error,
		inputClass = ''
	}: Props = $props();
</script>

<label class="label {className}">
	<span class="text-surface-600-300-token">{label}</span>

	{#if type === 'select'}
		<select class="select {inputClass}" bind:value {disabled} {required}>
			{#each options as option}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
	{:else if type === 'textarea'}
		<textarea
			class="textarea {inputClass}"
			bind:value
			{placeholder}
			{disabled}
			{required}
		></textarea>
	{:else if type === 'checkbox'}
		<input
			type="checkbox"
			class="checkbox {inputClass}"
			bind:checked={value}
			{disabled}
			{required}
		/>
	{:else}
		<input
			{type}
			class="input {inputClass}"
			bind:value
			{min}
			{max}
			{step}
			{placeholder}
			{disabled}
			{required}
		/>
	{/if}

	{#if error}
		<p class="text-error-500 text-sm mt-1">{error}</p>
	{:else if help}
		<p class="text-xs opacity-70 mt-1">{help}</p>
	{/if}
</label>
