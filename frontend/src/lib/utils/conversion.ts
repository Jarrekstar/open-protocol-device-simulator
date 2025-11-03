/**
 * Conversion utilities for rate and percentage transformations
 */

/**
 * Converts a decimal rate (0.0-1.0) to a percentage (0-100)
 * @param rate - The decimal rate to convert
 * @returns The percentage value
 * @example toPercentage(0.75) // returns 75
 */
export function toPercentage(rate: number): number {
	return rate * 100;
}

/**
 * Converts a percentage (0-100) to a decimal rate (0.0-1.0)
 * @param percentage - The percentage to convert
 * @returns The decimal rate
 * @example toRate(75) // returns 0.75
 */
export function toRate(percentage: number): number {
	return percentage / 100;
}

/**
 * Clamps a value between min and max
 * @param value - The value to clamp
 * @param min - The minimum value
 * @param max - The maximum value
 * @returns The clamped value
 */
export function clamp(value: number, min: number, max: number): number {
	return Math.min(Math.max(value, min), max);
}

/**
 * Rounds a number to a specified number of decimal places
 * @param value - The value to round
 * @param decimals - The number of decimal places
 * @returns The rounded value
 */
export function roundTo(value: number, decimals: number): number {
	const multiplier = Math.pow(10, decimals);
	return Math.round(value * multiplier) / multiplier;
}
