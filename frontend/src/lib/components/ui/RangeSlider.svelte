<script lang="ts">
	/**
	 * Dual-handle range slider component for selecting min/max values
	 * Provides visual feedback and keyboard accessibility
	 */
	interface Props {
		min: number; // Absolute minimum value
		max: number; // Absolute maximum value
		valueMin: number; // Current min selection
		valueMax: number; // Current max selection
		step?: number; // Step increment
		unit?: string; // Unit label (e.g., "Nm", "°")
		label?: string; // Field label
		disabled?: boolean;
		onValueChange?: (min: number, max: number) => void;
	}

	let {
		min = 0,
		max = 100,
		valueMin = $bindable(min),
		valueMax = $bindable(max),
		step = 1,
		unit = '',
		label = '',
		disabled = false,
		onValueChange
	}: Props = $props();

	let trackElement: HTMLDivElement | null = $state(null);
	let draggingHandle: 'min' | 'max' | null = $state(null);

	// Calculate positions as percentages
	const minPercent = $derived(((valueMin - min) / (max - min)) * 100);
	const maxPercent = $derived(((valueMax - min) / (max - min)) * 100);

	// Calculate decimal places for proper rounding
	const decimalPlaces = $derived(() => {
		const stepStr = step.toString();
		const decimalIndex = stepStr.indexOf('.');
		return decimalIndex === -1 ? 0 : stepStr.length - decimalIndex - 1;
	});

	// Round to proper decimal places to avoid floating point issues
	function roundToPrecision(value: number): number {
		const places = decimalPlaces();
		const multiplier = Math.pow(10, places);
		return Math.round(value * multiplier) / multiplier;
	}

	function handleMouseDown(handle: 'min' | 'max') {
		if (disabled) return;
		draggingHandle = handle;

		const handleMouseMove = (e: MouseEvent) => {
			if (!trackElement || !draggingHandle) return;

			const rect = trackElement.getBoundingClientRect();
			const percent = Math.min(Math.max(0, ((e.clientX - rect.left) / rect.width) * 100), 100);
			const rawValue = (percent / 100) * (max - min) + min;
			const steppedValue = Math.round(rawValue / step) * step;
			const newValue = roundToPrecision(Math.min(Math.max(min, steppedValue), max));

			if (draggingHandle === 'min') {
				// Ensure min doesn't exceed max
				valueMin = Math.min(newValue, valueMax - step);
			} else {
				// Ensure max doesn't go below min
				valueMax = Math.max(newValue, valueMin + step);
			}
		};

		const handleMouseUp = () => {
			draggingHandle = null;
			document.removeEventListener('mousemove', handleMouseMove);
			document.removeEventListener('mouseup', handleMouseUp);

			// Call onValueChange only once at the end of dragging
			if (onValueChange) {
				onValueChange(valueMin, valueMax);
			}
		};

		document.addEventListener('mousemove', handleMouseMove);
		document.addEventListener('mouseup', handleMouseUp);
	}

	function handleKeyDown(handle: 'min' | 'max', e: KeyboardEvent) {
		if (disabled) return;

		let delta = 0;
		if (e.key === 'ArrowLeft' || e.key === 'ArrowDown') {
			delta = -step;
		} else if (e.key === 'ArrowRight' || e.key === 'ArrowUp') {
			delta = step;
		} else {
			return;
		}

		e.preventDefault();

		if (handle === 'min') {
			valueMin = roundToPrecision(Math.min(Math.max(min, valueMin + delta), valueMax - step));
		} else {
			valueMax = roundToPrecision(Math.max(Math.min(max, valueMax + delta), valueMin + step));
		}

		if (onValueChange) {
			onValueChange(valueMin, valueMax);
		}
	}
</script>

<div class="space-y-3 {disabled ? 'opacity-50 cursor-not-allowed' : ''}">
	{#if label}
		<div class="flex items-center justify-between">
			<span class="text-sm font-medium text-surface-700 dark:text-surface-300">
				{label}
			</span>
			<div class="text-sm text-surface-600 dark:text-surface-400">
				<span class="font-semibold text-surface-900 dark:text-surface-100"
					>{valueMin.toFixed(decimalPlaces())}</span
				>
				–
				<span class="font-semibold text-surface-900 dark:text-surface-100"
					>{valueMax.toFixed(decimalPlaces())}</span
				>
				{#if unit}
					<span class="ml-1">{unit}</span>
				{/if}
			</div>
		</div>
	{/if}

	<!-- Slider Track -->
	<div
		bind:this={trackElement}
		class="relative h-2 rounded-full bg-surface-200 dark:bg-surface-300 {disabled
			? ''
			: 'cursor-pointer'}"
		role="group"
		aria-label={label || 'Range slider'}
	>
		<!-- Selected Range Bar -->
		<div
			class="absolute h-full rounded-full bg-gradient-to-r from-primary-400 to-primary-600"
			style="left: {minPercent}%; width: {maxPercent - minPercent}%"
		></div>

		<!-- Min Handle -->
		<div
			role="slider"
			class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-5 h-5 rounded-full border-2 border-surface-50 dark:border-surface-900 bg-primary-500 shadow-md hover:scale-110 focus:scale-110 focus:outline-none focus:ring-2 focus:ring-primary-400 focus:ring-offset-2 focus:ring-offset-surface-50 dark:focus:ring-offset-surface-900 {disabled
				? 'cursor-not-allowed'
				: 'cursor-grab active:cursor-grabbing'} {draggingHandle === 'min'
				? 'scale-110 ring-2 ring-primary-400'
				: ''}"
			style="left: {minPercent}%"
			onmousedown={() => handleMouseDown('min')}
			onkeydown={(e) => handleKeyDown('min', e)}
			tabindex={disabled ? -1 : 0}
			aria-label="Minimum value"
			aria-valuemin={min}
			aria-valuemax={max}
			aria-valuenow={valueMin}
			aria-disabled={disabled}
		></div>

		<!-- Max Handle -->
		<div
			role="slider"
			class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-5 h-5 rounded-full border-2 border-surface-50 dark:border-surface-900 bg-primary-500 shadow-md hover:scale-110 focus:scale-110 focus:outline-none focus:ring-2 focus:ring-primary-400 focus:ring-offset-2 focus:ring-offset-surface-50 dark:focus:ring-offset-surface-900 {disabled
				? 'cursor-not-allowed'
				: 'cursor-grab active:cursor-grabbing'} {draggingHandle === 'max'
				? 'scale-110 ring-2 ring-primary-400'
				: ''}"
			style="left: {maxPercent}%"
			onmousedown={() => handleMouseDown('max')}
			onkeydown={(e) => handleKeyDown('max', e)}
			tabindex={disabled ? -1 : 0}
			aria-label="Maximum value"
			aria-valuemin={min}
			aria-valuemax={max}
			aria-valuenow={valueMax}
			aria-disabled={disabled}
		></div>
	</div>

	<!-- Min/Max Labels -->
	<div class="flex items-center justify-between text-xs text-surface-500 dark:text-surface-400">
		<span>
			{min}{unit}
		</span>
		<span>
			{max}{unit}
		</span>
	</div>
</div>
