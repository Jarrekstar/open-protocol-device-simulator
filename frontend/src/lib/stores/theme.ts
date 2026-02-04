import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type Theme = 'light' | 'dark';

// Initialize from localStorage if available, default to light
const stored = browser ? localStorage.getItem('theme') : null;
const initialTheme: Theme = (stored as Theme) || 'light';

export const theme = writable<Theme>(initialTheme);

// Subscribe to theme changes and persist to localStorage + update document
theme.subscribe((value) => {
	if (browser) {
		localStorage.setItem('theme', value);

		// Update document class for Tailwind dark mode
		if (value === 'dark') {
			document.documentElement.classList.add('dark');
			document.body?.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
			document.body?.classList.remove('dark');
		}
	}
});

export function toggleTheme() {
	theme.update((current) => (current === 'light' ? 'dark' : 'light'));
}
