/**
 * Formatting utilities for display values
 */

/**
 * Formats a torque value with appropriate decimal places and unit
 * @param torque - The torque value in Nm
 * @param decimals - Number of decimal places (default: 2)
 * @returns Formatted torque string with unit
 * @example formatTorque(15.456) // returns "15.46 Nm"
 */
export function formatTorque(torque: number, decimals: number = 2): string {
	return `${torque.toFixed(decimals)} Nm`;
}

/**
 * Formats an angle value with appropriate decimal places and unit
 * @param angle - The angle value in degrees
 * @param decimals - Number of decimal places (default: 1)
 * @returns Formatted angle string with unit
 * @example formatAngle(45.67) // returns "45.7°"
 */
export function formatAngle(angle: number, decimals: number = 1): string {
	return `${angle.toFixed(decimals)}°`;
}

/**
 * Formats a percentage value
 * @param value - The percentage value
 * @param decimals - Number of decimal places (default: 1)
 * @returns Formatted percentage string
 * @example formatPercentage(75.456) // returns "75.5%"
 */
export function formatPercentage(value: number, decimals: number = 1): string {
	return `${value.toFixed(decimals)}%`;
}

/**
 * Formats a batch counter display
 * @param current - Current count
 * @param total - Total count
 * @returns Formatted batch string
 * @example formatBatchCounter(5, 10) // returns "5 / 10"
 */
export function formatBatchCounter(current: number, total: number): string {
	return `${current} / ${total}`;
}

/**
 * Formats a number with thousands separators
 * @param value - The number to format
 * @returns Formatted number string
 * @example formatNumber(1234567) // returns "1,234,567"
 */
export function formatNumber(value: number): string {
	return value.toLocaleString();
}

/**
 * Truncates a string to a maximum length with ellipsis
 * @param text - The text to truncate
 * @param maxLength - Maximum length before truncation
 * @returns Truncated string
 */
export function truncate(text: string, maxLength: number): string {
	if (text.length <= maxLength) return text;
	return text.slice(0, maxLength - 3) + '...';
}

/**
 * Formats a timestamp or date string to relative time
 * @param timestamp - The timestamp to format
 * @returns Relative time string (e.g., "2 minutes ago")
 */
export function formatRelativeTime(timestamp: Date | string): string {
	const date = timestamp instanceof Date ? timestamp : new Date(timestamp);
	const now = new Date();
	const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

	if (seconds < 60) return 'just now';
	if (seconds < 3600) return `${Math.floor(seconds / 60)} minutes ago`;
	if (seconds < 86400) return `${Math.floor(seconds / 3600)} hours ago`;
	return `${Math.floor(seconds / 86400)} days ago`;
}
