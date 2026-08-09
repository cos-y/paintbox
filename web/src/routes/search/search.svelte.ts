import { SURFACE_BITS, type FilterOptions, type SearchResult } from '$lib/paints.svelte';
import { stock } from '$lib/stock.svelte';
import { callWasm, WorkerCancelled } from '$lib/wasmClient';
import { untrack } from 'svelte';

const STORAGE_KEY = 'paintbox:search';

interface Serialized {
	selectedSeries: string[];
	surfaceTypes: string[];
	baseTypes: string[];
	searchScope: number;
	mixingLimit: number;
	model: number;
	/** 取色板上次颜色（hex），与 URL ?color= 互为备份 */
	color: number;
}

function load(): Serialized {
	try {
		if (typeof localStorage !== 'undefined') {
			const raw = localStorage.getItem(STORAGE_KEY);
			if (raw) return JSON.parse(raw);
		}
	} catch {}
	return {
		selectedSeries: [],
		surfaceTypes: [],
		baseTypes: [],
		searchScope: 0,
		mixingLimit: 0,
		model: 0,
		color: 0x18b9d5
	};
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

	persist() {
		untrack(() => {
			const data: Serialized = {
				selectedSeries: [...this.selectedSeries],
				surfaceTypes: this.surfaceTypes,
				baseTypes: this.baseTypes,
				searchScope: this.searchScope,
				mixingLimit: this.mixingLimit,
				model: this.model,
				color: this.color
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
				if (seq === this.seq && !(err instanceof WorkerCancelled)) {
					this.searching = false;
				}
			}
		}, 200);
	}
}

export const rt = new SearchRuntime();
