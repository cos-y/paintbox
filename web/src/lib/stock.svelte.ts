import { listPaints, paintId } from '$lib/paints.svelte';

export interface StockEntry {
	/** 持久化的唯一标识 `${brand}:${code}` */
	id: string;
	/** listPaints() 里的下标；不持久化，首次使用时从油漆数据懒解析 */
	index: number;
}

const STORAGE_KEY = 'paintbox:stock';

// 持久化只存 id（string[]），index 是运行时数据
const loadFromStorage = (): string[] => {
	if (typeof localStorage === 'undefined') return [];
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		return raw ? JSON.parse(raw) : [];
	} catch {
		return [];
	}
};

// id -> index 懒构建缓存。
// 注意：wasmReady 门控只保证组件渲染在 wasm 之后，但页面 chunk 的模块求值（顶层代码）
// 与 layout load 并行竞速，模块顶层调 listPaints() 会拿到未初始化的 wasm（实测报错）。
// 因此只在运行时（用户交互 / 遍历）才解析 index。
let indexById: Map<string, number> | null = null;
const ensureIndex = (): Map<string, number> => {
	if (indexById === null) {
		indexById = new Map(listPaints().map((p) => [paintId(p), p.index]));
	}
	return indexById;
};

class StockStore {
	/** 库存：key 是持久化的 id，value 含 listPaints 下标（不持久化） */
	owned = $state<Map<string, StockEntry>>(
		new Map(loadFromStorage().map((id) => [id, { id, index: -1 }]))
	);

	version = 0;

	has(id: string): boolean {
		return this.owned.has(id);
	}

	set(id: string, owned: boolean) {
		const next = new Map(this.owned);
		if (owned) {
			next.set(id, { id, index: ensureIndex().get(id) ?? -1 });
		} else {
			next.delete(id);
		}
		this.owned = next;
		this.persist();
		this.version += 1;
	}

	toggle(id: string) {
		this.set(id, !this.has(id));
	}

	/** 遍历所有库存条目（首次调用时解析 index） */
	*entries(): Generator<StockEntry> {
		const idx = ensureIndex();
		for (const e of this.owned.values()) {
			yield { id: e.id, index: idx.get(e.id) ?? -1 };
		}
	}

	private persist() {
		localStorage.setItem(STORAGE_KEY, JSON.stringify([...this.owned.keys()]));
	}
}

export const stock = new StockStore();
