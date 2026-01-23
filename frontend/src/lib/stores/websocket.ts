import { writable, get } from 'svelte/store';
import { deviceState } from './device';
import { addEvent } from './events';
import { addTighteningResult, autoTighteningProgress } from './tightening';
import { WEBSOCKET } from '$lib/config/constants';
import { getWebSocketUrl } from '$lib/config/env';
import { logger } from '$lib/utils';
import type { SimulatorEvent, DeviceState, MultiSpindleConfig, FailureConfig } from '$lib/types';

export const connected = writable(false);
export const reconnectAttempts = writable(0);

// Connection quality metrics
export const latency = writable(0); // Average message round-trip time in ms
export const packetLoss = writable(0); // Packet loss percentage (0-100)
export const connectionHealth = writable(100); // Overall connection health score (0-100)

let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

// Ping/pong latency tracking
let pingTimer: ReturnType<typeof setInterval> | null = null;
let lastPingTime: number = 0;
let latencyMeasurements: number[] = [];
const MAX_LATENCY_SAMPLES = 10;
const PING_INTERVAL_MS = 5000; // Send ping every 5 seconds

/**
 * Backend DeviceState interface matches what the API sends
 * Field names differ from frontend DeviceState interface
 */
interface BackendDeviceState {
	cell_id: number;
	channel_id: number;
	controller_name: string;
	supplier_code: string;
	tool_enabled: boolean;
	device_fsm_state: string; // Maps to tool_state in frontend
	vehicle_id: string | null; // Maps to vehicle_id_number in frontend
	current_job_id: number | null;
	current_pset_id: number | null;
	current_pset_name: string | null;
	multi_spindle_config: MultiSpindleConfig;
	failure_config: FailureConfig;
	tightening_tracker?: unknown;
}

/**
 * Type guard to check if data looks like a DeviceState object from backend
 * @param data - Data to validate
 * @returns True if data matches backend DeviceState structure
 */
function isDeviceState(data: unknown): data is BackendDeviceState {
	return (
		typeof data === 'object' &&
		data !== null &&
		'cell_id' in data &&
		'tool_enabled' in data &&
		'device_fsm_state' in data &&
		'current_pset_id' in data &&
		'multi_spindle_config' in data
	);
}

/**
 * Maps backend DeviceState to frontend DeviceState interface
 * Handles field name differences between backend and frontend
 */
function mapDeviceState(data: BackendDeviceState): DeviceState {
	return {
		cell_id: data.cell_id,
		channel_id: data.channel_id,
		controller_name: data.controller_name,
		tool_enabled: data.tool_enabled,
		tool_state: data.device_fsm_state, // Backend sends device_fsm_state, map to tool_state
		vehicle_id_number: data.vehicle_id ?? null, // Backend sends vehicle_id, map to vehicle_id_number
		current_job_id: data.current_job_id,
		current_pset_id: data.current_pset_id,
		current_pset_name: data.current_pset_name,
		multi_spindle_config: data.multi_spindle_config,
		failure_config: data.failure_config
	};
}

/**
 * Sends a ping message to measure round-trip latency
 * Records timestamp for RTT calculation when pong is received
 */
function sendPing() {
	if (!ws || ws.readyState !== WebSocket.OPEN) return;

	lastPingTime = Date.now();
	// Send a ping message (backend should echo or handle this)
	ws.send(JSON.stringify({ type: 'ping', timestamp: lastPingTime }));
}

/**
 * Handles pong response and calculates round-trip time
 * Updates latency metrics and connection health score
 */
function handlePong() {
	if (lastPingTime === 0) return;

	const now = Date.now();
	const rtt = now - lastPingTime; // Round-trip time in milliseconds

	// Store measurement
	latencyMeasurements.push(rtt);

	// Keep only last N samples
	if (latencyMeasurements.length > MAX_LATENCY_SAMPLES) {
		latencyMeasurements.shift();
	}

	// Calculate average latency
	const avgLatency =
		latencyMeasurements.reduce((sum, val) => sum + val, 0) / latencyMeasurements.length;
	latency.set(Math.round(avgLatency));

	// Calculate connection health based on real latency
	// Excellent: <50ms = 100%, Good: 50-200ms, Degraded: >200ms
	let latencyScore: number;
	if (avgLatency < 50) {
		latencyScore = 100;
	} else if (avgLatency < 200) {
		// Linear scale from 100% at 50ms to 60% at 200ms
		latencyScore = 100 - ((avgLatency - 50) / 150) * 40;
	} else {
		// Linear scale from 60% at 200ms to 0% at 1000ms
		latencyScore = Math.max(0, 60 - ((avgLatency - 200) / 800) * 60);
	}

	// Connection stability score based on reconnect attempts
	const reconnects = get(reconnectAttempts);
	const stabilityScore = Math.max(0, 100 - reconnects * 10);

	// Overall health is weighted average of latency and stability
	const health = Math.round(latencyScore * 0.7 + stabilityScore * 0.3);
	connectionHealth.set(health);

	// Packet loss simulation based on connection stability
	const loss = Math.min(100, reconnects * 2);
	packetLoss.set(loss);

	// Reset ping time
	lastPingTime = 0;
}

/**
 * Starts periodic ping messages to measure latency
 */
function startPingInterval() {
	// Clear any existing interval
	if (pingTimer) {
		clearInterval(pingTimer);
	}

	// Send initial ping
	sendPing();

	// Start periodic pings
	pingTimer = setInterval(sendPing, PING_INTERVAL_MS);
}

/**
 * Stops ping interval and cleans up
 */
function stopPingInterval() {
	if (pingTimer) {
		clearInterval(pingTimer);
		pingTimer = null;
	}
	lastPingTime = 0;
	latencyMeasurements = [];
}

/**
 * Type utility for creating a fully-typed event handler map
 * Each handler receives the exact event variant for its type key
 */
type EventHandlerMap = {
	[K in SimulatorEvent['type']]: (event: Extract<SimulatorEvent, { type: K }>) => void;
};

/**
 * Event handler map with automatic type narrowing
 * TypeScript enforces that all event types are handled
 */
const eventHandlers: EventHandlerMap = {
	TighteningCompleted: (event) => {
		addTighteningResult(event.result);
		addEvent(event);
	},
	ToolStateChanged: (event) => {
		deviceState.update((state) => {
			if (state) {
				state.tool_enabled = event.enabled;
			}
			return state;
		});
		addEvent(event);
	},
	AutoTighteningProgress: (event) => {
		autoTighteningProgress.set({
			counter: event.counter,
			target_size: event.target_size,
			running: event.running
		});
	},
	PsetChanged: (event) => {
		deviceState.update((state) => {
			if (state) {
				state.current_pset_id = event.pset_id;
				state.current_pset_name = event.pset_name;
			}
			return state;
		});
		addEvent(event);
	},
	VehicleIdChanged: (event) => {
		deviceState.update((state) => {
			if (state) {
				state.vehicle_id_number = event.vin;
			}
			return state;
		});
		addEvent(event);
	},
	MultiSpindleResultCompleted: (event) => {
		addEvent(event);
	},
	MultiSpindleStatusCompleted: (event) => {
		addEvent(event);
	},
	BatchCompleted: (event) => {
		addEvent(event);
	}
};

/**
 * Type-safe event dispatcher using discriminated union narrowing
 * Avoids unsafe type assertions by explicitly handling each event type
 */
function dispatchEvent(event: SimulatorEvent): void {
	switch (event.type) {
		case 'TighteningCompleted':
			eventHandlers.TighteningCompleted(event);
			break;
		case 'ToolStateChanged':
			eventHandlers.ToolStateChanged(event);
			break;
		case 'AutoTighteningProgress':
			eventHandlers.AutoTighteningProgress(event);
			break;
		case 'PsetChanged':
			eventHandlers.PsetChanged(event);
			break;
		case 'VehicleIdChanged':
			eventHandlers.VehicleIdChanged(event);
			break;
		case 'MultiSpindleResultCompleted':
			eventHandlers.MultiSpindleResultCompleted(event);
			break;
		case 'MultiSpindleStatusCompleted':
			eventHandlers.MultiSpindleStatusCompleted(event);
			break;
		case 'BatchCompleted':
			eventHandlers.BatchCompleted(event);
			break;
		default:
			// Exhaustiveness check - TypeScript will error if a case is missing
			const _exhaustive: never = event;
			logger.warn('Unknown event type received:', _exhaustive);
			return _exhaustive;
	}
}

/**
 * Establishes WebSocket connection with automatic reconnection
 * Uses exponential backoff with a maximum of 10 attempts
 * @param url - WebSocket endpoint URL (default: derived from environment or localhost:8081)
 */
export function connectWebSocket(url: string = getWebSocketUrl()) {
	// Prevent multiple instances
	if (ws?.readyState === WebSocket.OPEN || ws?.readyState === WebSocket.CONNECTING) {
		logger.info('WebSocket already connected or connecting');
		return;
	}

	// Clean up any existing connection
	if (ws) {
		ws.close();
		ws = null;
	}

	ws = new WebSocket(url);

	ws.onopen = () => {
		logger.info('WebSocket connected');
		connected.set(true);
		reconnectAttempts.set(0);

		// Reset metrics on new connection
		latency.set(0);
		packetLoss.set(0);
		connectionHealth.set(100);

		// Start ping/pong interval for latency measurement
		startPingInterval();
	};

	ws.onmessage = (event) => {
		try {
			const data = JSON.parse(event.data);

			// Check for pong response
			if (data.type === 'pong') {
				handlePong();
				return;
			}

			// First message might be DeviceState
			if (isDeviceState(data)) {
				deviceState.set(mapDeviceState(data));
				return;
			}

			// Otherwise it's a SimulatorEvent
			const simEvent = data as SimulatorEvent;

			// Route event to appropriate handler using type-safe dispatcher
			dispatchEvent(simEvent);
		} catch (error) {
			logger.error('Failed to parse WebSocket message:', error);
		}
	};

	ws.onerror = (error) => {
		logger.error('WebSocket error:', error);
	};

	ws.onclose = () => {
		logger.info('WebSocket disconnected');
		connected.set(false);

		// Stop ping interval
		stopPingInterval();

		// Reset metrics on disconnect
		connectionHealth.set(0);

		// Attempt to reconnect
		const attempts = get(reconnectAttempts);
		if (attempts < WEBSOCKET.MAX_RECONNECT_ATTEMPTS) {
			const delay = Math.min(
				WEBSOCKET.BASE_RECONNECT_DELAY_MS * Math.pow(2, attempts),
				WEBSOCKET.MAX_RECONNECT_DELAY_MS
			);
			logger.info(`Reconnecting in ${delay}ms (attempt ${attempts + 1})`);

			reconnectTimer = setTimeout(() => {
				reconnectAttempts.update((n) => n + 1);
				connectWebSocket(url);
			}, delay);
		}
	};
}

/**
 * Disconnects the WebSocket and cleans up resources
 * Cancels any pending reconnection attempts
 */
export function disconnectWebSocket() {
	// Clear reconnection timer
	if (reconnectTimer) {
		clearTimeout(reconnectTimer);
		reconnectTimer = null;
	}

	// Stop ping interval
	stopPingInterval();

	// Close and cleanup WebSocket
	if (ws) {
		// Remove event listeners to prevent memory leaks
		ws.onopen = null;
		ws.onmessage = null;
		ws.onerror = null;
		ws.onclose = null;

		ws.close();
		ws = null;
	}

	connected.set(false);
	reconnectAttempts.set(0);
}
