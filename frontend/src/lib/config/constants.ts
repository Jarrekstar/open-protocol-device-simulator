/**
 * Application Configuration Constants
 * Centralized constants for magic numbers and configuration values
 */

/**
 * Event and history management constants
 */
export const EVENTS = {
	/** Maximum number of events to keep in history */
	MAX_EVENTS: 1000,
	/** Maximum number of tightening results to keep in history */
	MAX_TIGHTENING_HISTORY: 100
} as const;

/**
 * WebSocket connection constants
 */
export const WEBSOCKET = {
	/** Maximum number of reconnection attempts before giving up */
	MAX_RECONNECT_ATTEMPTS: 10,
	/** Base delay for reconnection backoff (milliseconds) */
	BASE_RECONNECT_DELAY_MS: 1000,
	/** Maximum delay for reconnection backoff (milliseconds) */
	MAX_RECONNECT_DELAY_MS: 30000
} as const;

/**
 * Toast notification constants
 */
export const TOAST = {
	/** Default duration for toast notifications (milliseconds) */
	DEFAULT_DURATION_MS: 3000
} as const;
