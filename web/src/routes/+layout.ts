import type { LayoutLoad } from './$types';
import { browser } from '$app/environment';
import init, { init_searcher } from '../wasm-pkg/paintbox_wasm';
import { loadMeta } from '$lib/meta';
import { preloadPaintNames } from '$lib/i18ndyn.svelte';
import { initEquivs } from '$lib/equivs.svelte';
import { loadPaintExtras } from '$lib/paints.svelte';

export const ssr = false;
export const prerender = true;
// 让每个路由都prerender成 <route>/index.html 而不是 <route>.html：
// python http.server这类简单静态服务器只会把 /search 解析成 search/index.html，
// 不支持无扩展名解析成 search.html
export const trailingSlash = 'always';

export const load: LayoutLoad = async ({ fetch }) => {
	// ssr=false 页面 prerender 只生成空壳，node 端 load 无意义：
	// 客户端启动时（browser）会重新执行 load 获取数据
	if (!browser) return;

	// init_searcher 依赖 wasm 实例化完成，必须等 init() 之后再调用
	await init();
	const [paintsBuf, , eqBuf] = await Promise.all([
		fetch('/paints.bin').then((r) => r.arrayBuffer()),
		loadPaintExtras(fetch),
		fetch('/equivs.bin').then((r) => r.arrayBuffer()),
		loadMeta(fetch),
		preloadPaintNames(fetch)
	]);
	init_searcher(new Uint8Array(paintsBuf));
	initEquivs(eqBuf);
};
