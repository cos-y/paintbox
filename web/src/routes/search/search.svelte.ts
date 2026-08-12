import { SURFACE_BITS, type FilterOptions, type SearchResult } from '$lib/paints.svelte';
import { stock } from '$lib/stock.svelte';
import { loadData } from '$lib/utils.svelte';
import { callWasm, WorkerCancelled } from '$lib/wasmClient';
import { untrack } from 'svelte';
import { z } from 'zod';

const STORAGE_KEY = 'paintbox:search';

/** 展示面板模式：调色板 / 摄像机（仅 Tauri）/ 油漆 */
export type SourceMode = 'palette' | 'camera' | 'paint';

// 字符串数组字段：逐元素恢复——非字符串元素单独丢弃，合法元素保留
// （一个脏元素不连坐整个数组重置）
const strArray = () =>
	z
		.array(z.unknown())
		.transform((arr) => arr.filter((x): x is string => typeof x === 'string'))
		.catch([]);

// 持久化 schema：类型与校验单一来源（z.infer 推导 Serialized）。
// 历史字段缺省（24fb59c 6 字段 → 当前 9 字段）与类型错误（损坏/旧数据）
// 均由逐字段 .catch() 兜底为默认值，不会崩溃也不会互相污染。
const SerializedSchema = z.object({
	selectedSeries: strArray(),
	surfaceTypes: strArray(),
	baseTypes: strArray(),
	searchScope: z.number().catch(0),
	mixingLimit: z.number().catch(0),
	model: z.number().catch(0),
	/** 取色板上次颜色（hex），localStorage 持久化 */
	color: z.number().int().catch(0x18b9d5),
	/** 展示面板模式（持久化：刷新/重开后按模式直接进入对应面板） */
	source: z.enum(['palette', 'camera', 'paint']).catch('palette'),
	/** 油漆模式锚点：详情页「调配/查看全部」设置，油漆信息展示与颜色来源解析依据 */
	paintKey: z.string().nullable().catch(null)
});

type Serialized = z.infer<typeof SerializedSchema>;

// 只读一次持久化数据，供 SearchStore 各字段复用（避免每字段一次 load）
const initial = loadData(STORAGE_KEY, SerializedSchema);

class SearchStore {
	selectedSeries = $state<Set<string>>(new Set(initial.selectedSeries));
	surfaceTypes = $state<string[]>(initial.surfaceTypes);
	baseTypes = $state<string[]>(initial.baseTypes);
	searchScope = $state(initial.searchScope);
	mixingLimit = $state(initial.mixingLimit);
	model = $state(initial.model);
	color = $state<number>(initial.color);
	source = $state<SourceMode>(initial.source);
	paintKey = $state<string | null>(initial.paintKey);

	persist() {
		untrack(() => {
			const data: Serialized = {
				selectedSeries: [...this.selectedSeries],
				surfaceTypes: this.surfaceTypes,
				baseTypes: this.baseTypes,
				searchScope: this.searchScope,
				mixingLimit: this.mixingLimit,
				model: this.model,
				color: this.color,
				source: this.source,
				paintKey: this.paintKey
			};
			localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
		});
	}

	reset() {
		this.selectedSeries = new Set();
		this.surfaceTypes = [];
		this.baseTypes = [];
		this.searchScope = 0;
		this.mixingLimit = 0;
		this.color = 0x18b9d5;
		this.paintKey = null;
		this.source = 'palette';
		this.persist();
	}
}

export const store = new SearchStore();

// ---- 运行时搜索状态（不持久化）----
// 搜索结果是 (color, searchFilter, stock) 的纯函数：
// 模块级 $state 在页面间导航时保持；三个输入都不变时下面的 $effect 不会重跑，
// 因此切页再回来不会重新搜索。
//
// 注意：Svelte 不允许导出被重新赋值的模块级 $state（state_invalid_export），
// 所以用 class 实例包装（与上方 SearchStore 同一模式）。

class SearchRuntime {
	/** 取色板当前颜色（组件写入，rgb int） */
	results = $state<SearchResult[]>([]);
	searching = $state(false);

	/** 上次搜索的输入签名（color + filter + stock），相同则跳过重复搜索 */
	private lastSig = '';
	private seq = 0;
	private timer: ReturnType<typeof setTimeout> | undefined;

	private opts: FilterOptions = $derived.by(() => {
		const series = [...store.selectedSeries].map((key) => {
			const [brand, serie] = key.split('::');
			return [brand, serie];
		});
		const all = store.searchScope == 0 ? undefined : [...stock.entries()].map((e) => e.index);
		return {
			series,
			all,
			surfaces: store.surfaceTypes.reduce((m, k) => m | SURFACE_BITS[k], 0),
			bases: store.baseTypes.map((x) => +x),
			mix: store.mixingLimit,
			limit: 12
		};
	});

	/**
	 * 在组件 $effect 中调用。输入签名无变化时直接返回（切页回来不重搜）；
	 * 有变化则 debounce 后重新搜索。
	 */
	search() {
		// 先清理数据源中已失效的库存条目（编号更新后旧 id 不存在），避免 -1 传入 wasm
		const sig = JSON.stringify([store.color, this.opts, stock.version]);
		if (sig === this.lastSig) return;
		this.lastSig = sig;

		if (this.timer) clearTimeout(this.timer);
		const seq = ++this.seq;
		this.searching = true;
		this.timer = setTimeout(async () => {
			try {
				const r = await callWasm<SearchResult[] | null>('search', [store.color, this.opts], {
					cancelInFlight: true
				});
				if (seq === this.seq) {
					this.results = r ?? [];
					this.searching = false;
				}
			} catch (err) {
				console.error(err);
				if (seq === this.seq && !(err instanceof WorkerCancelled)) {
					this.searching = false;
				}
			}
		}, 200);
	}
}

export const rt = new SearchRuntime();
