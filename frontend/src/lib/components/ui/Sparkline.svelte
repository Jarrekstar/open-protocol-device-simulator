<script lang="ts">
	interface Props {
		data: number[];
		width?: number;
		height?: number;
		color?: string;
		filled?: boolean;
		showDot?: boolean;
		className?: string;
	}

	let {
		data = [],
		width = 100,
		height = 30,
		color = 'rgb(var(--color-primary-500))',
		filled = false,
		showDot = true,
		className = ''
	}: Props = $props();

	// Calculate min/max for normalization
	const minValue = $derived(data.length > 0 ? Math.min(...data) : 0);
	const maxValue = $derived(data.length > 0 ? Math.max(...data) : 0);
	const range = $derived(maxValue - minValue || 1);

	// Generate SVG path
	const pathData = $derived(() => {
		if (data.length === 0) return '';

		const points = data.map((value, index) => {
			const x = (index / (data.length - 1 || 1)) * width;
			const y = height - ((value - minValue) / range) * height;
			return { x, y };
		});

		const path = points
			.map((point, index) => {
				if (index === 0) {
					return `M ${point.x},${point.y}`;
				}
				return `L ${point.x},${point.y}`;
			})
			.join(' ');

		if (filled) {
			return `${path} L ${width},${height} L 0,${height} Z`;
		}

		return path;
	});

	const lastPoint = $derived(() => {
		if (data.length === 0) return null;
		const lastIndex = data.length - 1;
		const x = width;
		const y = height - ((data[lastIndex] - minValue) / range) * height;
		return { x, y };
	});
</script>

<svg
	{width}
	{height}
	class="inline-block {className}"
	viewBox="0 0 {width} {height}"
	preserveAspectRatio="none"
>
	{#if filled}
		<path
			d={pathData()}
			fill={color}
			opacity="0.2"
			stroke="none"
		/>
	{/if}
	<path
		d={pathData()}
		fill="none"
		stroke={color}
		stroke-width="2"
		stroke-linecap="round"
		stroke-linejoin="round"
	/>
	{#if showDot && lastPoint()}
		<circle
			cx={lastPoint()?.x}
			cy={lastPoint()?.y}
			r="3"
			fill={color}
		/>
	{/if}
</svg>
