import type { LayoutLoad } from './$types';
import init, { init_searcher } from '../wasm-pkg/paintbox_wasm';
import { loadMeta } from '$lib/meta';

export const ssr = false;
export const prerender = true;
// 让每个路由都prerender成 <route>/index.html 而不是 <route>.html：
// python http.server这类简单静态服务器只会把 /search 解析成 search/index.html，
// 不支持无扩展名解析成 search.html
export const trailingSlash = 'always';

export const load: LayoutLoad = async ({ fetch }) => {
	let [_, data, equivData] = await Promise.all([
		init(),
		fetch('/colors.csv').then((data) => data.arrayBuffer()),
		fetch('/equivalences.csv').then((data) => data.arrayBuffer()),
		loadMeta(fetch)
	]);
	init_searcher(new Uint8Array(data), new Uint8Array(equivData));
};
