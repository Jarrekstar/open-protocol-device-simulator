import { writable } from 'svelte/store';
import type { DeviceState } from '$lib/types';

/**
 * Central device state store
 * Updated by WebSocket events from the simulator
 */
export const deviceState = writable<DeviceState | null>(null);
