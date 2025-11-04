import { writable, get } from 'svelte/store';
import { deviceState } from './device';
import { addEvent } from './events';
import { addTighteningResult, autoTighteningProgress } from './tightening';
import { WEBSOCKET } from '$lib/config/constants';
import type { SimulatorEvent, DeviceState } from '$lib/types';

export const connected = writable(false);
export const reconnectAttempts = writable(0);

let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

/**
 * Type guard to check if data is a DeviceState object
 * @param data - Data to validate
 * @returns True if data matches DeviceState structure
 */
function isDeviceState(data: any): data is DeviceState {
	return (
		typeof data === 'object' &&
		data !== null &&
		typeof data.cell_id === 'number' &&
		typeof data.tool_enabled === 'boolean' &&
		'tool_state' in data &&
		'current_pset_id' in data &&
		'multi_spindle_config' in data &&
		'vehicle_id_number' in data
	);
}

/**
 * Event handler map for routing WebSocket events to appropriate store updates
 */
const eventHandlers: Record<string, (event: any) => void> = {
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
 * Establishes WebSocket connection with automatic reconnection
 * Uses exponential backoff with a maximum of 10 attempts
 * @param url - WebSocket endpoint URL (default: ws://localhost:8081/ws/events)
 */
export function connectWebSocket(url: string = 'ws://localhost:8081/ws/events') {
	// Prevent multiple instances
	if (ws?.readyState === WebSocket.OPEN || ws?.readyState === WebSocket.CONNECTING) {
		console.log('WebSocket already connected or connecting');
		return;
	}

	// Clean up any existing connection
	if (ws) {
		ws.close();
		ws = null;
	}

	ws = new WebSocket(url);

	ws.onopen = () => {
		console.log('WebSocket connected');
		connected.set(true);
		reconnectAttempts.set(0);
	};

	ws.onmessage = (event) => {
		try {
			const data = JSON.parse(event.data);

			// First message might be DeviceState
			if (isDeviceState(data)) {
				deviceState.set(data);
				return;
			}

			// Otherwise it's a SimulatorEvent
			const simEvent = data as SimulatorEvent;

			// Route event to appropriate handler
			if (simEvent.type && eventHandlers[simEvent.type]) {
				eventHandlers[simEvent.type](simEvent);
			} else {
				console.warn('Unknown event type received:', simEvent.type);
			}
		} catch (error) {
			console.error('Failed to parse WebSocket message:', error);
		}
	};

	ws.onerror = (error) => {
		console.error('WebSocket error:', error);
	};

	ws.onclose = () => {
		console.log('WebSocket disconnected');
		connected.set(false);

		// Attempt to reconnect
		const attempts = get(reconnectAttempts);
		if (attempts < WEBSOCKET.MAX_RECONNECT_ATTEMPTS) {
			const delay = Math.min(
				WEBSOCKET.BASE_RECONNECT_DELAY_MS * Math.pow(2, attempts),
				WEBSOCKET.MAX_RECONNECT_DELAY_MS
			);
			console.log(`Reconnecting in ${delay}ms (attempt ${attempts + 1})`);

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
