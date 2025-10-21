import { writable, get } from 'svelte/store';
import { deviceState } from './device';
import { addEvent } from './events';
import { addTighteningResult, autoTighteningProgress } from './tightening';
import type { SimulatorEvent } from '$lib/types';

export const connected = writable(false);
export const reconnectAttempts = writable(0);

let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

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

			// Route event to appropriate stores
			switch (simEvent.type) {
				case 'TighteningCompleted':
					addTighteningResult(simEvent.result);
					addEvent(simEvent);
					break;

				case 'ToolStateChanged':
					deviceState.update((state) => {
						if (state) {
							state.tool_enabled = simEvent.enabled;
						}
						return state;
					});
					addEvent(simEvent);
					break;

				case 'AutoTighteningProgress':
					autoTighteningProgress.set({
						counter: simEvent.counter,
						target_size: simEvent.target_size,
						running: simEvent.running
					});
					break;

				case 'PsetChanged':
					deviceState.update((state) => {
						if (state) {
							state.current_pset_id = simEvent.pset_id;
							state.current_pset_name = simEvent.pset_name;
						}
						return state;
					});
					addEvent(simEvent);
					break;

				case 'VehicleIdChanged':
					deviceState.update((state) => {
						if (state) {
							state.vehicle_id_number = simEvent.vin;
						}
						return state;
					});
					addEvent(simEvent);
					break;

				case 'MultiSpindleResultCompleted':
				case 'MultiSpindleStatusCompleted':
				case 'BatchCompleted':
					addEvent(simEvent);
					break;
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
