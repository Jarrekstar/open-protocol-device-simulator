import { writable } from 'svelte/store';

/**
 * Toast notification interface
 */
export interface Toast {
	id: string;
	type: 'success' | 'error' | 'info' | 'warning';
	message: string;
	duration?: number;
}

/**
 * Global toast notifications store
 */
export const toasts = writable<Toast[]>([]);

/**
 * Display a toast notification
 * @param toast - Toast configuration without id
 */
export function showToast(toast: Omit<Toast, 'id'>) {
	const id = crypto.randomUUID();
	const fullToast: Toast = { id, duration: 3000, ...toast };

	toasts.update((list) => [...list, fullToast]);

	if (fullToast.duration) {
		setTimeout(() => {
			dismissToast(id);
		}, fullToast.duration);
	}
}

/**
 * Dismiss a toast notification by id
 * @param id - Toast id to dismiss
 */
export function dismissToast(id: string) {
	toasts.update((list) => list.filter((t) => t.id !== id));
}
