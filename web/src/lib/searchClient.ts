import type { FilterOptions, SearchResult } from './paints';

let worker: Worker | null = null;
let nextId = 0;
const pending = new Map<number, (results: SearchResult[]) => void>();

const getWorker = (): Worker => {
	if (!worker) {
		worker = new Worker(new URL('./search.worker.ts', import.meta.url), { type: 'module' });
		worker.onmessage = (e: MessageEvent<{ id: number; results: SearchResult[] }>) => {
			const { id, results } = e.data;
			const resolve = pending.get(id);
			if (resolve) {
				pending.delete(id);
				resolve(results);
			}
		};
	}
	return worker;
};

export const searchAsync = (
	rgb: number,
	maxMix: number,
	limit: number,
	filter: FilterOptions
): Promise<SearchResult[]> => {
	const w = getWorker();
	const id = nextId++;
	return new Promise((resolve) => {
		pending.set(id, resolve);
		w.postMessage({ id, rgb, maxMix, limit, filter });
	});
};
