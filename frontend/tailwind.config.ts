// @ts-ignore
import { join } from 'path';
import type { Config } from 'tailwindcss';
import { skeleton } from '@skeletonlabs/tw-plugin';

const simulatorTheme = {
	name: 'simulator',
	properties: {
		'--theme-font-family-base':
			"'Inter', 'Segoe UI', 'Roboto', 'Helvetica Neue', -apple-system, BlinkMacSystemFont, sans-serif",
		'--theme-font-family-heading':
			"'Inter', 'Segoe UI', 'Roboto', 'Helvetica Neue', -apple-system, BlinkMacSystemFont, sans-serif",
		'--theme-font-color-base': '30 41 59',
		'--theme-font-color-dark': '255 255 255',
		'--theme-rounded-base': '12px',
		'--theme-rounded-container': '16px',
		'--theme-border-base': '1px',
		'--on-primary': '255 255 255',
		'--on-secondary': '255 255 255',
		'--on-tertiary': '255 255 255',
		'--on-success': '255 255 255',
		'--on-warning': '30 41 59',
		'--on-error': '255 255 255',
		'--on-surface': '30 41 59',
		// Primary: Refined Orange (Industrial/Energy)
		'--color-primary-50': '255 247 237',
		'--color-primary-100': '255 237 213',
		'--color-primary-200': '254 215 170',
		'--color-primary-300': '253 186 116',
		'--color-primary-400': '251 146 60',
		'--color-primary-500': '249 115 22',
		'--color-primary-600': '234 88 12',
		'--color-primary-700': '194 65 12',
		'--color-primary-800': '154 52 18',
		'--color-primary-900': '124 45 18',
		// Secondary: Deep Amber (Warm Accent)
		'--color-secondary-50': '254 252 232',
		'--color-secondary-100': '254 249 195',
		'--color-secondary-200': '254 240 138',
		'--color-secondary-300': '253 224 71',
		'--color-secondary-400': '250 204 21',
		'--color-secondary-500': '234 179 8',
		'--color-secondary-600': '202 138 4',
		'--color-secondary-700': '161 98 7',
		'--color-secondary-800': '133 77 14',
		'--color-secondary-900': '113 63 18',
		// Tertiary: Cool Blue (Technology/Precision)
		'--color-tertiary-50': '240 249 255',
		'--color-tertiary-100': '224 242 254',
		'--color-tertiary-200': '186 230 253',
		'--color-tertiary-300': '125 211 252',
		'--color-tertiary-400': '56 189 248',
		'--color-tertiary-500': '14 165 233',
		'--color-tertiary-600': '2 132 199',
		'--color-tertiary-700': '3 105 161',
		'--color-tertiary-800': '7 89 133',
		'--color-tertiary-900': '12 74 110',
		// Success: Vibrant Green
		'--color-success-50': '240 253 244',
		'--color-success-100': '220 252 231',
		'--color-success-200': '187 247 208',
		'--color-success-300': '134 239 172',
		'--color-success-400': '74 222 128',
		'--color-success-500': '34 197 94',
		'--color-success-600': '22 163 74',
		'--color-success-700': '21 128 61',
		'--color-success-800': '22 101 52',
		'--color-success-900': '20 83 45',
		// Warning: Amber
		'--color-warning-50': '255 251 235',
		'--color-warning-100': '254 243 199',
		'--color-warning-200': '253 230 138',
		'--color-warning-300': '252 211 77',
		'--color-warning-400': '251 191 36',
		'--color-warning-500': '245 158 11',
		'--color-warning-600': '217 119 6',
		'--color-warning-700': '180 83 9',
		'--color-warning-800': '146 64 14',
		'--color-warning-900': '120 53 15',
		// Error: Red
		'--color-error-50': '254 242 242',
		'--color-error-100': '254 226 226',
		'--color-error-200': '254 202 202',
		'--color-error-300': '252 165 165',
		'--color-error-400': '248 113 113',
		'--color-error-500': '239 68 68',
		'--color-error-600': '220 38 38',
		'--color-error-700': '185 28 28',
		'--color-error-800': '153 27 27',
		'--color-error-900': '127 29 29',
		// Surface: Soft Neutral Warm Grays (Light Mode)
		'--color-surface-50': '255 255 255',
		'--color-surface-100': '248 250 252',
		'--color-surface-200': '241 245 249',
		'--color-surface-300': '226 232 240',
		'--color-surface-400': '203 213 225',
		'--color-surface-500': '148 163 184',
		'--color-surface-600': '100 116 139',
		'--color-surface-700': '71 85 105',
		'--color-surface-800': '51 65 85',
		'--color-surface-900': '30 41 59'
	},
	properties_dark: {
		'--theme-font-family-base':
			"'Inter', 'Segoe UI', 'Roboto', 'Helvetica Neue', -apple-system, BlinkMacSystemFont, sans-serif",
		'--theme-font-family-heading':
			"'Inter', 'Segoe UI', 'Roboto', 'Helvetica Neue', -apple-system, BlinkMacSystemFont, sans-serif",
		'--theme-font-color-base': '255 255 255',
		'--theme-font-color-dark': '15 23 42',
		'--theme-rounded-base': '12px',
		'--theme-rounded-container': '16px',
		'--theme-border-base': '1px',
		'--on-primary': '255 255 255',
		'--on-secondary': '255 255 255',
		'--on-tertiary': '255 255 255',
		'--on-success': '255 255 255',
		'--on-warning': '15 23 42',
		'--on-error': '255 255 255',
		'--on-surface': '255 255 255',
		// Primary: Refined Orange (Industrial/Energy) - Slightly brighter for dark mode
		'--color-primary-50': '255 247 237',
		'--color-primary-100': '255 237 213',
		'--color-primary-200': '254 215 170',
		'--color-primary-300': '253 186 116',
		'--color-primary-400': '251 146 60',
		'--color-primary-500': '249 115 22',
		'--color-primary-600': '234 88 12',
		'--color-primary-700': '194 65 12',
		'--color-primary-800': '154 52 18',
		'--color-primary-900': '124 45 18',
		// Secondary: Deep Amber
		'--color-secondary-50': '254 252 232',
		'--color-secondary-100': '254 249 195',
		'--color-secondary-200': '254 240 138',
		'--color-secondary-300': '253 224 71',
		'--color-secondary-400': '250 204 21',
		'--color-secondary-500': '234 179 8',
		'--color-secondary-600': '202 138 4',
		'--color-secondary-700': '161 98 7',
		'--color-secondary-800': '133 77 14',
		'--color-secondary-900': '113 63 18',
		// Tertiary: Cool Blue (brighter for dark mode)
		'--color-tertiary-50': '240 249 255',
		'--color-tertiary-100': '224 242 254',
		'--color-tertiary-200': '186 230 253',
		'--color-tertiary-300': '125 211 252',
		'--color-tertiary-400': '56 189 248',
		'--color-tertiary-500': '14 165 233',
		'--color-tertiary-600': '2 132 199',
		'--color-tertiary-700': '3 105 161',
		'--color-tertiary-800': '7 89 133',
		'--color-tertiary-900': '12 74 110',
		// Success: Vibrant Green
		'--color-success-50': '240 253 244',
		'--color-success-100': '220 252 231',
		'--color-success-200': '187 247 208',
		'--color-success-300': '134 239 172',
		'--color-success-400': '74 222 128',
		'--color-success-500': '34 197 94',
		'--color-success-600': '22 163 74',
		'--color-success-700': '21 128 61',
		'--color-success-800': '22 101 52',
		'--color-success-900': '20 83 45',
		// Warning: Amber
		'--color-warning-50': '255 251 235',
		'--color-warning-100': '254 243 199',
		'--color-warning-200': '253 230 138',
		'--color-warning-300': '252 211 77',
		'--color-warning-400': '251 191 36',
		'--color-warning-500': '245 158 11',
		'--color-warning-600': '217 119 6',
		'--color-warning-700': '180 83 9',
		'--color-warning-800': '146 64 14',
		'--color-warning-900': '120 53 15',
		// Error: Red
		'--color-error-50': '254 242 242',
		'--color-error-100': '254 226 226',
		'--color-error-200': '254 202 202',
		'--color-error-300': '252 165 165',
		'--color-error-400': '248 113 113',
		'--color-error-500': '239 68 68',
		'--color-error-600': '220 38 38',
		'--color-error-700': '185 28 28',
		'--color-error-800': '153 27 27',
		'--color-error-900': '127 29 29',
		// Surface: Cool Slate Grays (Dark Mode) - Professional, neutral
		'--color-surface-50': '15 23 42',
		'--color-surface-100': '30 41 59',
		'--color-surface-200': '51 65 85',
		'--color-surface-300': '71 85 105',
		'--color-surface-400': '100 116 139',
		'--color-surface-500': '148 163 184',
		'--color-surface-600': '203 213 225',
		'--color-surface-700': '226 232 240',
		'--color-surface-800': '248 250 252',
		'--color-surface-900': '255 255 255'
	}
};

export default {
	darkMode: 'selector',
	content: [
		'./src/**/*.{html,js,svelte,ts}',
// @ts-ignore
		join(require.resolve('@skeletonlabs/skeleton'), '../**/*.{html,js,svelte,ts}')
	],
	theme: {
		extend: {
			colors: {
				primary: {
					50: 'rgb(var(--color-primary-50) / <alpha-value>)',
					100: 'rgb(var(--color-primary-100) / <alpha-value>)',
					200: 'rgb(var(--color-primary-200) / <alpha-value>)',
					300: 'rgb(var(--color-primary-300) / <alpha-value>)',
					400: 'rgb(var(--color-primary-400) / <alpha-value>)',
					500: 'rgb(var(--color-primary-500) / <alpha-value>)',
					600: 'rgb(var(--color-primary-600) / <alpha-value>)',
					700: 'rgb(var(--color-primary-700) / <alpha-value>)',
					800: 'rgb(var(--color-primary-800) / <alpha-value>)',
					900: 'rgb(var(--color-primary-900) / <alpha-value>)'
				},
				secondary: {
					50: 'rgb(var(--color-secondary-50) / <alpha-value>)',
					100: 'rgb(var(--color-secondary-100) / <alpha-value>)',
					200: 'rgb(var(--color-secondary-200) / <alpha-value>)',
					300: 'rgb(var(--color-secondary-300) / <alpha-value>)',
					400: 'rgb(var(--color-secondary-400) / <alpha-value>)',
					500: 'rgb(var(--color-secondary-500) / <alpha-value>)',
					600: 'rgb(var(--color-secondary-600) / <alpha-value>)',
					700: 'rgb(var(--color-secondary-700) / <alpha-value>)',
					800: 'rgb(var(--color-secondary-800) / <alpha-value>)',
					900: 'rgb(var(--color-secondary-900) / <alpha-value>)'
				},
				tertiary: {
					50: 'rgb(var(--color-tertiary-50) / <alpha-value>)',
					100: 'rgb(var(--color-tertiary-100) / <alpha-value>)',
					200: 'rgb(var(--color-tertiary-200) / <alpha-value>)',
					300: 'rgb(var(--color-tertiary-300) / <alpha-value>)',
					400: 'rgb(var(--color-tertiary-400) / <alpha-value>)',
					500: 'rgb(var(--color-tertiary-500) / <alpha-value>)',
					600: 'rgb(var(--color-tertiary-600) / <alpha-value>)',
					700: 'rgb(var(--color-tertiary-700) / <alpha-value>)',
					800: 'rgb(var(--color-tertiary-800) / <alpha-value>)',
					900: 'rgb(var(--color-tertiary-900) / <alpha-value>)'
				},
				success: {
					50: 'rgb(var(--color-success-50) / <alpha-value>)',
					100: 'rgb(var(--color-success-100) / <alpha-value>)',
					200: 'rgb(var(--color-success-200) / <alpha-value>)',
					300: 'rgb(var(--color-success-300) / <alpha-value>)',
					400: 'rgb(var(--color-success-400) / <alpha-value>)',
					500: 'rgb(var(--color-success-500) / <alpha-value>)',
					600: 'rgb(var(--color-success-600) / <alpha-value>)',
					700: 'rgb(var(--color-success-700) / <alpha-value>)',
					800: 'rgb(var(--color-success-800) / <alpha-value>)',
					900: 'rgb(var(--color-success-900) / <alpha-value>)'
				},
				warning: {
					50: 'rgb(var(--color-warning-50) / <alpha-value>)',
					100: 'rgb(var(--color-warning-100) / <alpha-value>)',
					200: 'rgb(var(--color-warning-200) / <alpha-value>)',
					300: 'rgb(var(--color-warning-300) / <alpha-value>)',
					400: 'rgb(var(--color-warning-400) / <alpha-value>)',
					500: 'rgb(var(--color-warning-500) / <alpha-value>)',
					600: 'rgb(var(--color-warning-600) / <alpha-value>)',
					700: 'rgb(var(--color-warning-700) / <alpha-value>)',
					800: 'rgb(var(--color-warning-800) / <alpha-value>)',
					900: 'rgb(var(--color-warning-900) / <alpha-value>)'
				},
				error: {
					50: 'rgb(var(--color-error-50) / <alpha-value>)',
					100: 'rgb(var(--color-error-100) / <alpha-value>)',
					200: 'rgb(var(--color-error-200) / <alpha-value>)',
					300: 'rgb(var(--color-error-300) / <alpha-value>)',
					400: 'rgb(var(--color-error-400) / <alpha-value>)',
					500: 'rgb(var(--color-error-500) / <alpha-value>)',
					600: 'rgb(var(--color-error-600) / <alpha-value>)',
					700: 'rgb(var(--color-error-700) / <alpha-value>)',
					800: 'rgb(var(--color-error-800) / <alpha-value>)',
					900: 'rgb(var(--color-error-900) / <alpha-value>)'
				},
				surface: {
					50: 'rgb(var(--color-surface-50) / <alpha-value>)',
					100: 'rgb(var(--color-surface-100) / <alpha-value>)',
					200: 'rgb(var(--color-surface-200) / <alpha-value>)',
					300: 'rgb(var(--color-surface-300) / <alpha-value>)',
					400: 'rgb(var(--color-surface-400) / <alpha-value>)',
					500: 'rgb(var(--color-surface-500) / <alpha-value>)',
					600: 'rgb(var(--color-surface-600) / <alpha-value>)',
					700: 'rgb(var(--color-surface-700) / <alpha-value>)',
					800: 'rgb(var(--color-surface-800) / <alpha-value>)',
					900: 'rgb(var(--color-surface-900) / <alpha-value>)'
				}
			},
			// Enhanced spacing system (8px grid)
			spacing: {
				'18': '4.5rem',
				'88': '22rem',
				'128': '32rem'
			},
			// Typography scale with letter-spacing
			fontSize: {
				'2xs': ['0.625rem', { lineHeight: '0.875rem', letterSpacing: '0.01em' }],
				xs: ['0.75rem', { lineHeight: '1rem', letterSpacing: '0.01em' }],
				sm: ['0.875rem', { lineHeight: '1.25rem', letterSpacing: '0.005em' }],
				base: ['1rem', { lineHeight: '1.5rem', letterSpacing: 'normal' }],
				lg: ['1.125rem', { lineHeight: '1.75rem', letterSpacing: '-0.01em' }],
				xl: ['1.25rem', { lineHeight: '1.75rem', letterSpacing: '-0.01em' }],
				'2xl': ['1.5rem', { lineHeight: '2rem', letterSpacing: '-0.015em' }],
				'3xl': ['1.875rem', { lineHeight: '2.25rem', letterSpacing: '-0.02em' }],
				'4xl': ['2.25rem', { lineHeight: '2.5rem', letterSpacing: '-0.025em' }]
			},
			// Enhanced box shadows for depth and elevation
			boxShadow: {
				'sm': '0 1px 2px 0 rgb(0 0 0 / 0.05)',
				'DEFAULT': '0 2px 8px rgb(var(--color-primary-500) / 0.08)',
				'md': '0 4px 12px rgb(var(--color-primary-500) / 0.12)',
				'lg': '0 8px 24px rgb(var(--color-primary-500) / 0.15)',
				'xl': '0 16px 48px rgb(var(--color-primary-500) / 0.18)',
				'glow': '0 0 20px rgb(var(--color-primary-400) / 0.4)',
				'glow-success': '0 0 20px rgb(var(--color-success-500) / 0.4)',
				'glow-error': '0 0 20px rgb(var(--color-error-500) / 0.4)',
				'inner': 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)'
			},
			// Animation and transition presets
			transitionDuration: {
				'250': '250ms',
				'350': '350ms'
			},
			transitionTimingFunction: {
				'smooth': 'cubic-bezier(0.4, 0, 0.2, 1)',
				'bounce-in': 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
				'ease-out-expo': 'cubic-bezier(0.16, 1, 0.3, 1)'
			},
			// Animation keyframes
			keyframes: {
				'pulse-glow': {
					'0%, 100%': { opacity: '1', boxShadow: '0 0 8px rgb(var(--color-primary-400) / 0.3)' },
					'50%': { opacity: '0.8', boxShadow: '0 0 16px rgb(var(--color-primary-400) / 0.6)' }
				},
				'slide-up': {
					'0%': { transform: 'translateY(10px)', opacity: '0' },
					'100%': { transform: 'translateY(0)', opacity: '1' }
				},
				'slide-down': {
					'0%': { transform: 'translateY(-10px)', opacity: '0' },
					'100%': { transform: 'translateY(0)', opacity: '1' }
				},
				'scale-in': {
					'0%': { transform: 'scale(0.95)', opacity: '0' },
					'100%': { transform: 'scale(1)', opacity: '1' }
				},
				'shimmer': {
					'0%': { backgroundPosition: '-1000px 0' },
					'100%': { backgroundPosition: '1000px 0' }
				}
			},
			animation: {
				'pulse-glow': 'pulse-glow 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
				'slide-up': 'slide-up 0.3s ease-out-expo',
				'slide-down': 'slide-down 0.3s ease-out-expo',
				'scale-in': 'scale-in 0.2s ease-out',
				'shimmer': 'shimmer 2s linear infinite'
			}
		}
	},
	plugins: [
		skeleton({
			themes: {
				preset: [
					{
						name: 'skeleton',
						enhancements: true
					},
					{
						name: 'modern',
						enhancements: true
					}
				],
				custom: [simulatorTheme]
			}
		})
	]
} satisfies Config;
