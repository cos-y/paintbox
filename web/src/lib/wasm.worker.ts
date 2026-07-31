import init, * as wasm from '../wasm-pkg/paintbox_wasm';

// 通用 wasm RPC worker：client 通过方法名调用 wasm 导出的任意函数。
//
// 只有登记在 METHODS 白名单里的方法可被调用，避免 client 触达无关导出。
// wasm 调用是同步阻塞的，一旦跑起来无法从内部叫停；client 侧负责在卡住时
// terminate 整个 worker 重开。

interface CallRequest {
	id: number;
	method: string;
	args: unknown[];
}

interface CallResponse {
	id: number;
	ok: boolean;
	value?: unknown;
	error?: string;
}

// 需要一次性初始化的准备工作（wasm 实例化 + 数据加载）
let ready: Promise<void> | null = null;

const ensureReady = (): Promise<void> => {
	if (!ready) {
		ready = (async () => {
			const [, buf, equivBuf] = await Promise.all([
				init(),
				fetch('/colors.csv').then((r) => r.arrayBuffer()),
				fetch('/equivalences.csv').then((r) => r.arrayBuffer())
			]);
			wasm.init_searcher(new Uint8Array(buf), new Uint8Array(equivBuf));
		})();
	}
	return ready;
};

const OBJECTS = new Map<string, any>();
const allocObject = (obj: any) => {
	const uuid = self.crypto.randomUUID();
	OBJECTS.set(uuid, obj);
	return uuid;
};

const freeObject = (uuid: string) => {
	const obj = OBJECTS.get(uuid);
	obj?.free();
	OBJECTS.delete(uuid);
};

function getObject<T>(uuid: string) {
	return OBJECTS.get(uuid) as T;
}

// 可被 client 调用的 wasm 方法白名单。新增方法在这里登记即可。
const METHODS: Record<string, (...args: never[]) => unknown> = {
	free: (uuid: string) => freeObject(uuid),
	search: wasm.search,
	new_gamut: (ndiv: number, li: Uint32Array) => allocObject(wasm.new_gamut(ndiv, li)),
	gamut_insert_many: (gamut: string, rgbs: Uint32Array) =>
		getObject<wasm.HullProxy>(gamut).insert_many(rgbs),
	gamut_matrices: (gamut: string) => getObject<wasm.HullProxy>(gamut).matrices(),
	gamut_colors: (gamut: string) => getObject<wasm.HullProxy>(gamut).colors()
	// 在此登记更多 wasm 导出方法
};

self.onmessage = async (e: MessageEvent<CallRequest>) => {
	const { id, method, args } = e.data;

	let response: CallResponse;
	try {
		await ensureReady();
		const fn = METHODS[method];
		if (!fn) {
			throw new Error(`unknown wasm method: ${method}`);
		}
		const value = fn(...(args as never[]));
		response = { id, ok: true, value };
	} catch (err) {
		response = { id, ok: false, error: err instanceof Error ? err.message : String(err) };
	}

	(self as unknown as Worker).postMessage(response);
};
