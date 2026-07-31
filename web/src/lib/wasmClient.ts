// 通用 wasm 异步调用客户端。
//
// 在 worker 里跑 wasm，避免阻塞主线程。wasm 调用同步且无法从内部叫停，
// 所以当有请求还在执行时又来了新请求（说明 worker 正卡在耗时调用里），
// 直接 terminate 整个 worker 重开，比等它跑完更快。

interface PendingCall {
	resolve: (value: unknown) => void;
	reject: (err: Error) => void;
}

interface CallResponse {
	id: number;
	ok: boolean;
	value?: unknown;
	error?: string;
}

export interface CallOptions {
	/** 发起前若有未完成的请求，是否终止并重启 worker。默认 true。 */
	cancelInFlight?: boolean;
}

/** worker 被主动终止时，未完成请求收到的错误。 */
export class WorkerCancelled extends Error {
	constructor() {
		super('wasm worker cancelled');
		this.name = 'WorkerCancelled';
	}
}

let worker: Worker | null = null;
let nextId = 0;
const pending = new Map<number, PendingCall>();

const spawnWorker = (): Worker => {
	const w = new Worker(new URL('./wasm.worker.ts', import.meta.url), { type: 'module' });
	w.onmessage = (e: MessageEvent<CallResponse>) => {
		const { id, ok, value, error } = e.data;
		const p = pending.get(id);
		if (!p) return;
		pending.delete(id);
		if (ok) {
			p.resolve(value);
		} else {
			p.reject(new Error(error ?? 'wasm call failed'));
		}
	};
	w.onerror = (event) => {
		console.error(event);
	};
	return w;
};

const getWorker = (): Worker => {
	if (!worker) {
		worker = spawnWorker();
	}
	return worker;
};

// 终止 worker 并 reject 所有未完成请求，避免它们永久挂起
const terminateWorker = () => {
	if (!worker) return;
	worker.terminate();
	worker = null;
	for (const p of pending.values()) {
		p.reject(new WorkerCancelled());
	}
	pending.clear();
};

/**
 * 异步调用一个 wasm 方法（须在 worker 的 METHODS 白名单中登记）。
 * @param method 方法名
 * @param args   传给 wasm 函数的参数
 * @param opts   调用选项
 */
export const callWasm = <T = unknown>(
	method: string,
	args: unknown[] = [],
	opts: CallOptions = {}
): Promise<T> => {
	const { cancelInFlight = false } = opts;
	if (cancelInFlight && pending.size > 0) {
		terminateWorker();
	}
	const w = getWorker();
	const id = nextId++;
	// console.log(method, args);
	return new Promise<T>((resolve, reject) => {
		pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
		w.postMessage({ id, method, args });
	});
};

/** 手动终止 worker（例如页面卸载时），未完成请求会被 reject。 */
export const cancelWasm = () => {
	terminateWorker();
};
