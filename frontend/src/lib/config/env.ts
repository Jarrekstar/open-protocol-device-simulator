/**
 * Environment configuration helpers for the frontend.
 *
 * These functions provide centralized access to environment-based configuration,
 * with sensible defaults for local development.
 */

/**
 * Gets the base URL for the REST API.
 *
 * Uses VITE_API_BASE_URL environment variable if set,
 * otherwise defaults to localhost:8081 for development.
 *
 * @returns The API base URL (e.g., 'http://localhost:8081')
 */
export function getApiBaseUrl(): string {
	return import.meta.env.VITE_API_BASE_URL || 'http://localhost:8081';
}

/**
 * Gets the WebSocket URL for real-time events.
 *
 * Priority:
 * 1. VITE_WS_URL environment variable (if set)
 * 2. Derived from VITE_API_BASE_URL (converts http(s) to ws(s))
 * 3. Default: ws://localhost:8081/ws/events
 *
 * @returns The WebSocket URL (e.g., 'ws://localhost:8081/ws/events')
 */
export function getWebSocketUrl(): string {
	const wsUrl = import.meta.env.VITE_WS_URL;
	if (wsUrl) return wsUrl;

	const apiBase = getApiBaseUrl();
	const wsProtocol = apiBase.startsWith('https') ? 'wss' : 'ws';
	return apiBase.replace(/^https?/, wsProtocol) + '/ws/events';
}
