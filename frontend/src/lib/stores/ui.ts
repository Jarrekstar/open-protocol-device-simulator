import { writable } from 'svelte/store';

export interface Toast {
	id: string;
	type: 'success' | 'error' | 'info' | 'warning';
	message: string;
	duration?: number;
}

export const toasts = writable<Toast[]>([]);
export const activeModal = writable<string | null>(null);

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

export function dismissToast(id: string) {
	toasts.update((list) => list.filter((t) => t.id !== id));
}

export function openModal(modalId: string) {
	activeModal.set(modalId);
}

export function closeModal() {
	activeModal.set(null);
}
