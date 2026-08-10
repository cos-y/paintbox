import { SURFACE_BITS, type FilterOptions, type SearchResult } from '$lib/paints.svelte';
import { stock } from '$lib/stock.svelte';
import { callWasm, WorkerCancelled } from '$lib/wasmClient';
import { untrack } from 'svelte';

const STORAGE_KEY = 'paintbox:search';

/** 展示面板模式：调色板 / 摄像机（仅 Tauri）/ 油漆 */
export type SourceMode = 'palette' | 'camera' | 'paint';

interface Serialized {
	selectedSeries: string[];
	surfaceTypes: string[];
	baseTypes: string[];
	searchScope: number;
	mixingLimit: number;
	model: number;
	/** 取色板上次颜色（hex），localStorage 持久化 */
	color: number;
	/** 展示面板模式（持久化：刷新/重开后按模式直接进入对应面板） */
	source: SourceMode;
	/** 油漆模式锚点：详情页「调配/查看全部」设置，油漆信息展示与颜色来源解析依据 */
	paintKey: string | null;
}

const DEFAULT: Serialized = {
	selectedSeries: [],
	surfaceTypes: [],
	baseTypes: [],
	searchScope: 0,
	mixingLimit: 0,
	model: 0,
	color: 0x18b9d5,
	source: 'palette',
	paintKey: null
};

function load(): Serialized {
	try {
		if (typeof localStorage !== 'undefined') {
			const raw = localStorage.getItem(STORAGE_KEY);
			// 与默认值合并：兼容旧版本缺省字段的持久化数据
			if (raw) return { ...DEFAULT, ...JSON.parse(raw) };
		}
	} catch {}
	return { ...DEFAULT };
}

// 只读一次持久化数据，供 SearchStore 各字段复用（避免每字段一次 load）
const initial = load();

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
