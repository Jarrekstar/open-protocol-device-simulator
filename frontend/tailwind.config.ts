// @ts-ignore
import { join } from 'path';
import type { Config } from 'tailwindcss';
import { skeleton } from '@skeletonlabs/tw-plugin';

const simulatorTheme = {
	name: 'simulator',
	properties: {
		'--theme-font-family-base': '"Sora", "Segoe UI", sans-serif',
		'--theme-font-family-heading': '"Space Grotesk", "Segoe UI", sans-serif',
		'--theme-font-color-base': '17 21 26',
		'--theme-font-color-dark': '12 15 18',
		'--theme-rounded-base': '16px',
		'--theme-rounded-container': '24px',
		'--theme-border-base': '1px',
		'--on-primary': '12 15 18',
		'--on-secondary': '12 15 18',
		'--on-tertiary': '12 15 18',
		'--on-success': '12 15 18',
		'--on-warning': '12 15 18',
		'--on-error': '255 255 255',
		'--on-surface': '17 21 26',
		// Primary: Ember
		'--color-primary-50': '255 245 236',
		'--color-primary-100': '255 233 212',
		'--color-primary-200': '255 215 179',
		'--color-primary-300': '255 208 166',
		'--color-primary-400': '255 176 121',
		'--color-primary-500': '255 154 98',
		'--color-primary-600': '255 146 94',
		'--color-primary-700': '229 120 71',
		'--color-primary-800': '198 101 56',
		'--color-primary-900': '168 84 45',
		// Secondary: Steel
		'--color-secondary-50': '244 248 251',
		'--color-secondary-100': '227 237 243',
		'--color-secondary-200': '200 219 230',
		'--color-secondary-300': '170 198 215',
		'--color-secondary-400': '155 186 208',
		'--color-secondary-500': '122 162 184',
		'--color-secondary-600': '95 135 157',
		'--color-secondary-700': '74 108 128',
		'--color-secondary-800': '56 82 99',
		'--color-secondary-900': '38 56 69',
		// Tertiary: Steel (cool accent)
		'--color-tertiary-50': '244 248 251',
		'--color-tertiary-100': '227 237 243',
		'--color-tertiary-200': '200 219 230',
		'--color-tertiary-300': '170 198 215',
		'--color-tertiary-400': '155 186 208',
		'--color-tertiary-500': '122 162 184',
		'--color-tertiary-600': '95 135 157',
		'--color-tertiary-700': '74 108 128',
		'--color-tertiary-800': '56 82 99',
		'--color-tertiary-900': '38 56 69',
		// Success: Mint
		'--color-success-50': '238 252 248',
		'--color-success-100': '214 247 236',
		'--color-success-200': '182 238 220',
		'--color-success-300': '143 228 203',
		'--color-success-400': '117 221 193',
		'--color-success-500': '111 226 194',
		'--color-success-600': '77 207 174',
		'--color-success-700': '47 178 147',
		'--color-success-800': '31 143 120',
		'--color-success-900': '22 109 92',
		// Warning: Warm amber
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
		// Error: Crimson
		'--color-error-50': '255 241 241',
		'--color-error-100': '255 214 214',
		'--color-error-200': '255 183 183',
		'--color-error-300': '255 148 148',
		'--color-error-400': '255 122 122',
		'--color-error-500': '255 107 107',
		'--color-error-600': '242 79 79',
		'--color-error-700': '217 58 58',
		'--color-error-800': '184 46 46',
		'--color-error-900': '143 35 35',
		// Surface: Light graphite scale
		'--color-surface-50': '246 247 249',
		'--color-surface-100': '238 241 244',
		'--color-surface-200': '225 230 235',
		'--color-surface-300': '205 212 221',
		'--color-surface-400': '174 183 196',
		'--color-surface-500': '141 153 168',
		'--color-surface-600': '109 122 137',
		'--color-surface-700': '79 89 102',
		'--color-surface-800': '47 55 66',
		'--color-surface-900': '17 21 26'
	},
	properties_dark: {
		'--theme-font-family-base': '"Sora", "Segoe UI", sans-serif',
		'--theme-font-family-heading': '"Space Grotesk", "Segoe UI", sans-serif',
		'--theme-font-color-base': '241 245 249',
		'--theme-font-color-dark': '12 15 18',
		'--theme-rounded-base': '16px',
		'--theme-rounded-container': '24px',
		'--theme-border-base': '1px',
		'--on-primary': '12 15 18',
		'--on-secondary': '12 15 18',
		'--on-tertiary': '12 15 18',
		'--on-success': '12 15 18',
		'--on-warning': '12 15 18',
		'--on-error': '255 255 255',
		'--on-surface': '241 245 249',
		// Primary: Ember
		'--color-primary-50': '255 245 236',
		'--color-primary-100': '255 233 212',
		'--color-primary-200': '255 215 179',
		'--color-primary-300': '255 208 166',
		'--color-primary-400': '255 176 121',
		'--color-primary-500': '255 154 98',
		'--color-primary-600': '255 146 94',
		'--color-primary-700': '229 120 71',
		'--color-primary-800': '198 101 56',
		'--color-primary-900': '168 84 45',
		// Secondary: Steel
		'--color-secondary-50': '244 248 251',
		'--color-secondary-100': '227 237 243',
		'--color-secondary-200': '200 219 230',
		'--color-secondary-300': '170 198 215',
		'--color-secondary-400': '155 186 208',
		'--color-secondary-500': '122 162 184',
		'--color-secondary-600': '95 135 157',
		'--color-secondary-700': '74 108 128',
		'--color-secondary-800': '56 82 99',
		'--color-secondary-900': '38 56 69',
		// Tertiary: Steel (cool accent)
		'--color-tertiary-50': '244 248 251',
		'--color-tertiary-100': '227 237 243',
		'--color-tertiary-200': '200 219 230',
		'--color-tertiary-300': '170 198 215',
		'--color-tertiary-400': '155 186 208',
		'--color-tertiary-500': '122 162 184',
		'--color-tertiary-600': '95 135 157',
		'--color-tertiary-700': '74 108 128',
		'--color-tertiary-800': '56 82 99',
		'--color-tertiary-900': '38 56 69',
		// Success: Mint
		'--color-success-50': '238 252 248',
		'--color-success-100': '214 247 236',
		'--color-success-200': '182 238 220',
		'--color-success-300': '143 228 203',
		'--color-success-400': '117 221 193',
		'--color-success-500': '111 226 194',
		'--color-success-600': '77 207 174',
		'--color-success-700': '47 178 147',
		'--color-success-800': '31 143 120',
		'--color-success-900': '22 109 92',
		// Warning: Warm amber
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
		// Error: Crimson
		'--color-error-50': '255 241 241',
		'--color-error-100': '255 214 214',
		'--color-error-200': '255 183 183',
		'--color-error-300': '255 148 148',
		'--color-error-400': '255 122 122',
		'--color-error-500': '255 107 107',
		'--color-error-600': '242 79 79',
		'--color-error-700': '217 58 58',
		'--color-error-800': '184 46 46',
		'--color-error-900': '143 35 35',
		// Surface: Dark graphite scale
		'--color-surface-50': '12 15 18',
		'--color-surface-100': '17 21 26',
		'--color-surface-200': '22 27 33',
		'--color-surface-300': '30 36 43',
		'--color-surface-400': '43 51 61',
		'--color-surface-500': '58 69 82',
		'--color-surface-600': '81 96 113',
		'--color-surface-700': '122 139 158',
		'--color-surface-800': '169 183 198',
		'--color-surface-900': '241 245 249'
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
				},
				graphite: {
					50: '#f6f7f9',
					100: '#eceff2',
					200: '#d6dce2',
					300: '#bcc6cf',
					400: '#98a3af',
					500: '#6e7a88',
					600: '#2b333d',
					700: '#1e242b',
					800: '#161b21',
					900: '#11151a',
					950: '#0c0f12'
				},
				ember: {
					50: '#fff5ec',
					100: '#ffe9d4',
					200: '#ffd7b3',
					300: '#ffd0a6',
					400: '#ffb079',
					500: '#ff9a62',
					600: '#ff925e',
					700: '#e57847',
					800: '#c66538',
					900: '#a8542d'
				},
				steel: {
					50: '#f4f8fb',
					100: '#e3edf3',
					200: '#c8dbe6',
					300: '#aac6d7',
					400: '#9bbad0',
					500: '#7aa2b8',
					600: '#5f879d',
					700: '#4a6c80',
					800: '#385263',
					900: '#263845'
				},
				mint: {
					50: '#eefcf8',
					100: '#d6f7ec',
					200: '#b6eedc',
					300: '#8fe4cb',
					400: '#75ddc1',
					500: '#6fe2c2',
					600: '#4dcfae',
					700: '#2fb293',
					800: '#1f8f78',
					900: '#166d5c'
				},
				crimson: {
					50: '#fff1f1',
					100: '#ffd6d6',
					200: '#ffb7b7',
					300: '#ff9494',
					400: '#ff7a7a',
					500: '#ff6b6b',
					600: '#f24f4f',
					700: '#d93a3a',
					800: '#b82e2e',
					900: '#8f2323'
				}
			},
			backgroundImage: {
				'ember-radial': 'radial-gradient(45% 45% at 20% 10%, rgba(255, 154, 98, 0.35), transparent 60%)',
				'cool-radial': 'radial-gradient(35% 35% at 80% 0%, rgba(111, 226, 194, 0.25), transparent 60%)',
				'deep-radial': 'radial-gradient(60% 60% at 50% 100%, rgba(122, 162, 184, 0.15), transparent 60%)',
				'ember-radial-soft': 'radial-gradient(45% 45% at 20% 10%, rgba(255, 154, 98, 0.18), transparent 65%)',
				'cool-radial-soft': 'radial-gradient(35% 35% at 80% 0%, rgba(111, 226, 194, 0.12), transparent 65%)',
				'deep-radial-soft': 'radial-gradient(60% 60% at 50% 100%, rgba(122, 162, 184, 0.1), transparent 65%)'
			},
			fontFamily: {
				display: ['"Space Grotesk"', '"Segoe UI"', 'sans-serif'],
				body: ['"Sora"', '"Segoe UI"', 'sans-serif']
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
				sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
				DEFAULT: '0 2px 8px rgb(12 15 18 / 0.08)',
				md: '0 4px 12px rgb(12 15 18 / 0.12)',
				lg: '0 8px 24px rgb(12 15 18 / 0.16)',
				xl: '0 16px 48px rgb(12 15 18 / 0.2)',
				glow: '0 0 35px rgba(255, 154, 98, 0.25)',
				'glow-soft': '0 0 22px rgba(255, 154, 98, 0.15)',
				'cool-glow': '0 0 28px rgba(122, 162, 184, 0.2)',
				'glow-success': '0 0 20px rgba(111, 226, 194, 0.35)',
				'glow-error': '0 0 20px rgba(255, 107, 107, 0.35)',
				inner: 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)'
			},
			// Animation and transition presets
			transitionDuration: {
				'250': '250ms',
				'350': '350ms'
			},
			transitionTimingFunction: {
				smooth: 'cubic-bezier(0.4, 0, 0.2, 1)',
				'bounce-in': 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
				'ease-out-expo': 'cubic-bezier(0.16, 1, 0.3, 1)'
			},
			// Animation keyframes
			keyframes: {
				'pulse-glow': {
					'0%, 100%': { opacity: '1', boxShadow: '0 0 8px rgba(255, 154, 98, 0.3)' },
					'50%': { opacity: '0.8', boxShadow: '0 0 16px rgba(255, 154, 98, 0.6)' }
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
				shimmer: {
					'0%': { backgroundPosition: '-1000px 0' },
					'100%': { backgroundPosition: '1000px 0' }
				}
			},
			animation: {
				'pulse-glow': 'pulse-glow 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
				'slide-up': 'slide-up 0.3s ease-out-expo',
				'slide-down': 'slide-down 0.3s ease-out-expo',
				'scale-in': 'scale-in 0.2s ease-out',
				shimmer: 'shimmer 2s linear infinite'
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
