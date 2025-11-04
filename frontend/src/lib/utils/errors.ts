/**
 * Error Handling Utilities
 * Safe extraction of error messages for user display
 */

/**
 * Extracts a user-friendly error message from various error types
 * @param error - The error to process (can be Error, string, or unknown)
 * @returns A safe, user-friendly error message
 */
export function getErrorMessage(error: unknown): string {
	if (error instanceof Error) {
		return error.message;
	}

	if (typeof error === 'string') {
		return error;
	}

	if (error && typeof error === 'object' && 'message' in error) {
		return String((error as any).message);
	}

	return 'An unexpected error occurred. Please try again.';
}

/**
 * Formats an error message with a contextual prefix
 * @param operation - The operation that failed (e.g., "load PSETs", "save configuration")
 * @param error - The error that occurred
 * @returns Formatted error message
 */
export function formatErrorMessage(operation: string, error: unknown): string {
	const message = getErrorMessage(error);
	return `Failed to ${operation}: ${message}`;
}
