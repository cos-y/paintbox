// 通用 wasm 异步调用客户端。
//
// 在 worker 里跑 wasm，避免阻塞主线程。wasm 调用同步且无法从内部叫停，
// 所以当有请求还在执行时又来了新请求（说明 worker 正卡在耗时调用里），
// 直接 terminate 整个 worker 重开，比等它跑完更快。
//
// ── Rust 对象生命周期约定（wasm.worker.ts 与各 .svelte.ts 共同遵守）──
// 1. 创建：调用以 `::new` 结尾的方法时在 wasm 侧创建 Rust 对象，返回句柄 uuid。
//    client 端 callWasm 见到返回值自动注册到 objs，此后 isValidObject(uuid) 为 true。
// 2. 销毁：调用 `::free`（方法名，第一个参数是句柄 uuid）或 wasm worker 被重启时
//    销毁对象。client 端在 `::free` 发送前注销该句柄，terminateWorker 时清空全部。
// 3. 跨页面：从别的页面切换进来时，用 .svelte.ts 模块级单例保存的句柄前，
//    必须先 isValidObject(uuid) 检查——期间可能发生过 worker 重启，句柄已失效。
// 4. 页面内部状态转移：若操作带 cancelInFlight（会 terminate 整个 worker），
//    旧句柄全部失效，调用方须自行重建对象（如 gamut.svelte.ts 的 rebuild）。

interface PendingCall {
	workerId: number;
	method: string;
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
let workerId = 0;
let requestId = 0;

const pending = new Map<number, PendingCall>();
const objs = new Set<string>();

const spawnWorker = (): Worker => {
	const myWorkerId = ++workerId;
	const w = new Worker(new URL('./wasm.worker.ts', import.meta.url), { type: 'module' });
	w.onmessage = (e: MessageEvent<CallResponse>) => {
		const { id, ok, value, error } = e.data;
		const p = pending.get(id);
		if (!p) return;
		pending.delete(id);
		if (ok) {
			if (workerId == myWorkerId) {
				if (p.method.endsWith('::new')) {
					let uuid = value as string;
					objs.add(uuid);
					console.log(`${p.method}: ${uuid}`);
				}
				p.resolve(value);
			} else {
				// 响应来自已被替换的旧 worker：对象不可用，按取消处理，避免 promise 永久挂起
				p.reject(new WorkerCancelled());
			}
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
	objs.clear();
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
		console.warn(`wasm call "${method}" cancelled ${pending.size} in-flight requests`);
	}
	const w = getWorker();
	const id = requestId++;
	// console.log(method, args);
	// 销毁型调用：发送前先在 client 侧注销句柄（对象在 worker 侧随 free 销毁）。
	// 即使 free 在 worker 侧失败，client 也按失效处理——宁可重建，不可用失效句柄。
	if (method == '::free') {
		let uuid = args[0] as string;
		if (isValidObject(uuid)) {
			objs.delete(uuid);
			console.log(`::free: ${uuid}`);
		}
	}
	return new Promise<T>((resolve, reject) => {
		pending.set(id, {
			workerId,
			method,
			resolve: resolve as (v: unknown) => void,
			reject
		});
		w.postMessage({ id, method, args });
	});
};

/** 手动终止 worker（例如页面卸载时），未完成请求会被 reject。 */
export const cancelWasm = () => {
	terminateWorker();
};

export const isValidObject = (uuid: string) => {
	return objs.has(uuid);
};
