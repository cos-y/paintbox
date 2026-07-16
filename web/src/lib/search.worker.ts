import init, { init_searcher, search } from '../wasm-pkg/paintbox_wasm';
import type { FilterOptions, SearchResult } from './paints';

interface SearchRequest {
	id: number;
	rgb: number;
	maxMix: number;
	limit: number;
	filter: FilterOptions;
}

interface SearchResponse {
	id: number;
	results: SearchResult[];
}

let ready: Promise<void> | null = null;

const ensureReady = (): Promise<void> => {
	if (!ready) {
		ready = (async () => {
			const [, buf, equivBuf] = await Promise.all([
				init(),
				fetch('/colors.csv').then((r) => r.arrayBuffer()),
				fetch('/equivalences.csv').then((r) => r.arrayBuffer())
			]);
			init_searcher(new Uint8Array(buf), new Uint8Array(equivBuf));
		})();
	}
	return ready;
};

self.onmessage = async (e: MessageEvent<SearchRequest>) => {
	const { id, rgb, maxMix, limit, filter } = e.data;
	await ensureReady();
	const results = (search(rgb, maxMix, limit, filter) as SearchResult[]) ?? [];
	const response: SearchResponse = { id, results };
	(self as unknown as Worker).postMessage(response);
};
