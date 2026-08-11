import { getPaintById } from '$lib/paints.svelte';

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

class StockStore {
	/** 库存：key 是持久化的 id，value 含 listPaints 下标（不持久化） */
	values = $state<Set<string>>(new Set(loadFromStorage()));
	version = 0;

	has(id: string): boolean {
		return this.values.has(id);
	}

	set(id: string, value: boolean) {
		if (this.values.has(id) === value) return;
		// $state 不代理 Set/Map，原地 add/delete 不会触发响应式更新；必须整体替换
		const next = new Set(this.values);
		if (value) {
			next.add(id);
		} else {
			next.delete(id);
		}
		this.values = next;
		this.persist();
		this.version += 1;
	}

	toggle(id: string) {
		this.set(id, !this.has(id));
	}

	/**
	 * 遍历所有库存条目（index 从数据源实时解析；
	 * 数据源中已不存在的条目自动跳过，由 prune() 负责持久化清理）
	 */
	*entries(): Generator<StockEntry> {
		let toRemove = [];
		for (const id of this.values.values()) {
			const { index } = getPaintById(id) ?? {};
			if (index !== undefined) {
				yield { id, index };
			} else {
				toRemove.push(id);
			}
		}
		if (toRemove.length > 0) {
			const next = new Set(this.values);
			for (const id of toRemove) {
				next.delete(id);
			}
			this.values = next;
			this.persist();
			this.version += 1;
		}
	}

	private persist() {
		localStorage.setItem(STORAGE_KEY, JSON.stringify([...this.values.keys()]));
	}
}

export const stock = new StockStore();
