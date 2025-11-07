<script lang="ts">
	interface Props {
		label: string;
		value: string;
		active?: boolean;
		onRemove?: (value: string) => void;
		onClick?: (value: string) => void;
		variant?: 'default' | 'primary' | 'success' | 'warning' | 'error';
		removable?: boolean;
	}

	let {
		label,
		value,
		active = false,
		onRemove,
		onClick,
		variant = 'default',
		removable = true
	}: Props = $props();

	const variantClasses = {
		default: active
			? 'bg-surface-300 dark:bg-surface-400 border-surface-400 dark:border-surface-500'
			: 'bg-surface-100 dark:bg-surface-200 border-surface-300 dark:border-surface-400',
		primary: active
			? 'bg-primary-100 dark:bg-primary-900 border-primary-400 dark:border-primary-600 text-primary-700 dark:text-primary-300'
			: 'bg-primary-50 dark:bg-primary-900/30 border-primary-200 dark:border-primary-800 text-primary-600 dark:text-primary-400',
		success: active
			? 'bg-success-100 dark:bg-success-900 border-success-400 dark:border-success-600 text-success-700 dark:text-success-300'
			: 'bg-success-50 dark:bg-success-900/30 border-success-200 dark:border-success-800 text-success-600 dark:text-success-400',
		warning: active
			? 'bg-warning-100 dark:bg-warning-900 border-warning-400 dark:border-warning-600 text-warning-700 dark:text-warning-300'
			: 'bg-warning-50 dark:bg-warning-900/30 border-warning-200 dark:border-warning-800 text-warning-600 dark:text-warning-400',
		error: active
			? 'bg-error-100 dark:bg-error-900 border-error-400 dark:border-error-600 text-error-700 dark:text-error-300'
			: 'bg-error-50 dark:bg-error-900/30 border-error-200 dark:border-error-800 text-error-600 dark:text-error-400'
	};

	function handleClick() {
		if (onClick) {
			onClick(value);
		}
	}

	function handleRemove(e: MouseEvent) {
		e.stopPropagation();
		if (onRemove) {
			onRemove(value);
		}
	}
</script>

<div
	class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full border text-sm font-medium transition-all duration-200 hover:scale-105 {variantClasses[
		variant
	]} {onClick ? 'cursor-pointer hover:shadow-md' : ''}"
	role="button"
	tabindex="0"
	onclick={handleClick}
	onkeydown={(e) => e.key === 'Enter' && handleClick()}
	aria-label="{active ? 'Active filter' : 'Inactive filter'}: {label}"
>
	<span>{label}</span>
	{#if removable && onRemove}
		<button
			type="button"
			class="ml-0.5 hover:bg-surface-400/30 dark:hover:bg-surface-600/30 rounded-full p-0.5 transition-colors"
			onclick={handleRemove}
			aria-label="Remove {label} filter"
		>
			<svg
				class="w-3.5 h-3.5"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M6 18L18 6M6 6l12 12"
				/>
			</svg>
		</button>
	{/if}
</div>
