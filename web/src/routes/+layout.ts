import type { LayoutLoad } from './$types';
import init, { init_searcher } from '../wasm-pkg/paintbox_wasm';
import { loadMeta } from '$lib/meta';

export const ssr = false;
export const prerender = true;

export const load: LayoutLoad = async ({ fetch }) => {
	let [_, data, equivData] = await Promise.all([
		init(),
		fetch('/colors.csv').then((data) => data.arrayBuffer()),
		fetch('/equivalences.csv').then((data) => data.arrayBuffer()),
		loadMeta(fetch)
	]);
	init_searcher(new Uint8Array(data), new Uint8Array(equivData));
};
