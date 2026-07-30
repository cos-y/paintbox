import type { FilterOptions, SearchResult } from './paints';
import { callWasm, WorkerCancelled } from './wasmClient';

// search 的薄封装，保持原有接口。底层走通用 wasm RPC 客户端：
// 新请求会取消（terminate）仍在执行的旧请求。被取消的请求这里静默返回空数组，
// 维持旧行为（旧实现里被取消的 Promise 直接丢弃、不 resolve）。
export const searchAsync = async (
	rgb: number,
	opts: FilterOptions
): Promise<SearchResult[]> => {
	try {
		const results = await callWasm<SearchResult[] | null>('search', [rgb, opts]);
		return results ?? [];
	} catch (err) {
		if (err instanceof WorkerCancelled) {
			// 被更新的请求抢占，返回空，让调用方忽略这次结果
			return [];
		}
		throw err;
	}
};
