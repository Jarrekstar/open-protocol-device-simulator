import { writable, get } from 'svelte/store';
import { deviceState } from './device';
import { addEvent } from './events';
import { addTighteningResult, autoTighteningProgress } from './tightening';
import type { SimulatorEvent } from '$lib/types';

export const connected = writable(false);
export const reconnectAttempts = writable(0);

let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

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

export function connectWebSocket(url: string = 'ws://localhost:8081/ws/events') {
	if (ws?.readyState === WebSocket.OPEN) {
		console.log('WebSocket already connected');
		return;
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
			if ('cell_id' in data && 'tool_enabled' in data) {
				deviceState.set(data);
				return;
			}

			// Otherwise it's a SimulatorEvent
			const simEvent = data as SimulatorEvent;

			// Route event to appropriate handler
			const handler = eventHandlers[simEvent.type];
			if (handler) {
				handler(simEvent);
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
		if (attempts < 10) {
			const delay = Math.min(1000 * Math.pow(2, attempts), 30000); // Exponential backoff, max 30s
			console.log(`Reconnecting in ${delay}ms (attempt ${attempts + 1})`);

			reconnectTimer = setTimeout(() => {
				reconnectAttempts.update((n) => n + 1);
				connectWebSocket(url);
			}, delay);
		}
	};
}

export function disconnectWebSocket() {
	if (reconnectTimer) {
		clearTimeout(reconnectTimer);
		reconnectTimer = null;
	}

	if (ws) {
		ws.close();
		ws = null;
	}

	connected.set(false);
}
