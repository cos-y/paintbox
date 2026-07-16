import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) => filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},

			// 纯静态输出：全站ssr=false+prerender=true、没有动态路由段，
			// 所以每个路由都能在build时prerender成一个真实的html文件，
			// 不需要fallback（python http.server这类简单静态服务器也不支持SPA fallback重写）
			adapter: adapter({
				pages: 'build',
				assets: 'build',
				fallback: undefined,
				strict: true
			})
		})
	]
});
