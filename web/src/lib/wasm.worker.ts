import init, * as wasm from '../wasm-pkg/paintbox_wasm';

// 通用 wasm RPC worker：client 通过方法名调用 wasm 导出的任意函数。
//
// 只有登记在 METHODS 白名单里的方法可被调用，避免 client 触达无关导出。
// wasm 调用是同步阻塞的，一旦跑起来无法从内部叫停；client 侧负责在卡住时
// terminate 整个 worker 重开。
//
// ── Rust 对象生命周期约定（镜像 wasmClient.ts 的 objs 注册表）──
// 1. `::new` 结尾的方法：创建 Rust 对象，allocObject 登记进 OBJECTS 并返回 uuid。
// 2. `::free`：从 OBJECTS 取出对象调用其 .free() 并注销。
// 3. worker 被 terminate 重启：OBJECTS 整体清空（新 worker 是全新状态），
//    client 侧 isValidObject 随即返回 false，调用方须重建对象。
// 4. 持有句柄的方法（如 gamut_insert_many）必须经 getObject 取对象；
//    失效句柄（对象不存在）直接抛明确错误，不要静默返回 undefined。

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
			const [, buf] = await Promise.all([
				init(),
				fetch('/paints.bin').then((r) => r.arrayBuffer())
			]);
			wasm.init_searcher(new Uint8Array(buf));
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
	const obj = OBJECTS.get(uuid);
	if (obj === undefined) {
		throw new Error(`wasm object ${uuid} not found (stale handle after worker restart)`);
	}
	return obj as T;
}

// 可被 client 调用的 wasm 方法白名单。新增方法在这里登记即可。
const METHODS: Record<string, (...args: never[]) => unknown> = {
	'::free': (uuid: string) => freeObject(uuid),
	search: wasm.search,
	'gamut::new': (ndiv: number, li: Uint32Array) => allocObject(wasm.new_gamut(ndiv, li)),
	gamut_insert_many: (gamut: string, rgbs: Uint32Array) =>
		getObject<wasm.Gamut>(gamut).insert_many(rgbs),
	gamut_matrices: (gamut: string) => getObject<wasm.Gamut>(gamut).matrices(),
	gamut_colors: (gamut: string) => getObject<wasm.Gamut>(gamut).colors(),
	// 散点模式：一次性计算，拷贝返回后释放 wasm 对象（view 是借用，free 前必须拷贝）
	scatter: (ndiv: number, colors: Uint32Array) => {
		const out = wasm.scatter(ndiv, colors);
		const data = {
			matrices: out.matrices().slice(),
			colors: out.colors().slice(),
			members: out.members().slice(),
			offsets: out.offsets().slice()
		};
		out.free();
		return data;
	}
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
