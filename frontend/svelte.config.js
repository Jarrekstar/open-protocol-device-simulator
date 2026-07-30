import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

const buildDir = process.env.WEBUI_BUILD_DIR ?? 'build';
const kitOutDir = process.env.SVELTE_KIT_OUT_DIR ?? '.svelte-kit';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter({
			pages: buildDir,
			assets: buildDir
		}),
		outDir: kitOutDir
	}
};

export default config;
