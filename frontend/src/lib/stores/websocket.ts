import { writable, get } from 'svelte/store';
import { deviceState } from './device';
import { addEvent } from './events';
import { addTighteningResult, autoTighteningProgress } from './tightening';
import { WEBSOCKET } from '$lib/config/constants';
import { logger } from '$lib/utils';
import type { SimulatorEvent, DeviceState, MultiSpindleConfig, FailureConfig } from '$lib/types';

export const connected = writable(false);
export const reconnectAttempts = writable(0);

let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

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
		typeof (data as BackendDeviceState).cell_id === 'number' &&
		'tool_enabled' in data &&
		typeof (data as BackendDeviceState).tool_enabled === 'boolean' &&
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
}

/**
 * Establishes WebSocket connection with automatic reconnection
 * Uses exponential backoff with a maximum of 10 attempts
 * @param url - WebSocket endpoint URL (default: ws://localhost:8081/ws/events)
 */
export function connectWebSocket(url: string = 'ws://localhost:8081/ws/events') {
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
	};

	ws.onmessage = (event) => {
		try {
			const data = JSON.parse(event.data);

			// First message might be DeviceState
			if (isDeviceState(data)) {
				deviceState.set(mapDeviceState(data));
				return;
			}

			// Otherwise it's a SimulatorEvent
			const simEvent = data as SimulatorEvent;

			// Route event to appropriate handler using handler map
			const handler = eventHandlers[simEvent.type];
			if (handler) {
				// Type assertion required due to TypeScript limitation with correlated union types
				(handler as (event: SimulatorEvent) => void)(simEvent);
			} else {
				logger.warn('Unknown event type received:', simEvent);
			}
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
