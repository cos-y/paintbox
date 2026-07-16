import type { FilterOptions, SearchResult } from './paints';

let worker: Worker | null = null;
let nextId = 0;
const pending = new Map<number, (results: SearchResult[]) => void>();

const spawnWorker = (): Worker => {
	const w = new Worker(new URL('./search.worker.ts', import.meta.url), { type: 'module' });
	w.onmessage = (e: MessageEvent<{ id: number; results: SearchResult[] }>) => {
		const { id, results } = e.data;
		const resolve = pending.get(id);
		if (resolve) {
			pending.delete(id);
			resolve(results);
		}
	};
	return w;
};

// wasm的search是同步阻塞的，一旦跑起来就没法从内部叫停；如果上一次请求还没返回
// 新请求就来了，说明worker正卡在耗时的旧搜索里（比如混色数调太大），直接把整个
// worker终止掉重开一个，比等它跑完再换参数快得多
const cancelInFlight = () => {
	if (worker && pending.size > 0) {
		worker.terminate();
		worker = null;
		pending.clear();
	}
};

const getWorker = (): Worker => {
	if (!worker) {
		worker = spawnWorker();
	}
	return worker;
};

export const searchAsync = (
	rgb: number,
	maxMix: number,
	limit: number,
	filter: FilterOptions
): Promise<SearchResult[]> => {
	cancelInFlight();
	const w = getWorker();
	const id = nextId++;
	return new Promise((resolve) => {
		pending.set(id, resolve);
		w.postMessage({ id, rgb, maxMix, limit, filter });
	});
};
