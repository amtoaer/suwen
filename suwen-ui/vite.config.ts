import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit(), tailwindcss()],
	server: {
		port: 5545,
		proxy: {
			'/api': {
				target: process.env.ORIGIN || 'http://localhost:4545',
				changeOrigin: true
			}
		}
	}
});
