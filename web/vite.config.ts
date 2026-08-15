import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { resolve } from 'path';
import { defineConfig } from 'vite';
import { minifyStaticJson } from './plugins/minifyStaticJson.ts';
import { version } from './package.json' with { type: 'json' };

export default defineConfig({
	define: {
		__APP_VERSION__: JSON.stringify(version),
		// 渠道标识：CI 打包时通过 CHANNEL 环境变量注入（play / 空=sideload / 未来 huawei 等）
		__CHANNEL__: JSON.stringify(process.env.CHANNEL ?? '')
	},
	resolve: {
		alias: {
			// 只导出本项目用到的符号，避免 extras barrel 连带 GLTF/DRACOLoader
			'@threlte/extras': resolve(import.meta.dirname, 'src/lib/threlte-extras-shim.ts')
		}
	},
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},

			// 纯静态输出：全站ssr=false+prerender=true、没有动态路由段，
			// 所以每个路由都能在build时prerender成一个真实的html文件，
			// 不需要fallback（python http.server这类简单静态服务器也不支持SPA fallback重写）
			adapter: adapter({
				pages: 'build',
				assets: 'build',
				fallback: 'index.html',
				strict: true
			})
		}),
		// 放在 sveltekit 之后：closeBundle 按注册顺序执行，确保 adapter 已写入 build/
		minifyStaticJson()
	]
});
