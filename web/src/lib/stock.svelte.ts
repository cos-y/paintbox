import { getPaintById } from '$lib/paints.svelte';
import { z } from 'zod';
import { loadData } from './utils.svelte';

export interface StockEntry {
	/** 持久化的唯一标识 `${brand}:${code}` */
	id: string;
	/** listPaints() 里的下标；不持久化，首次使用时从油漆数据懒解析 */
	index: number;
}

const STORAGE_KEY = 'paintbox:stock';

// 持久化格式：string[]（0.2.3 起所有版本）。
// 逐元素恢复：数组内非字符串元素单独丢弃，合法 id 保留（一个脏元素不连坐整个库存）；
// 整体非数组 / 损坏 JSON → catch([]) 重置，杜绝 new Set(非 iterable) 崩溃。
const StockSchema = z
	.array(z.unknown())
	.transform((arr) => arr.filter((x): x is string => typeof x === 'string'))
	.catch([]);

const initial = loadData(STORAGE_KEY, StockSchema);

class StockStore {
	/** 库存：key 是持久化的 id，value 含 listPaints 下标（不持久化） */
	values = $state<Set<string>>(new Set(initial));
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
	 * 整体替换库存（备份导入用）：
	 * 相同内容时跳过，避免无谓的持久化写入
	 */
	replaceAll(ids: Iterable<string>) {
		const next = new Set(ids);
		const same =
			next.size === this.values.size && [...next].every((v) => this.values.has(v));
		if (same) return;
		this.values = next;
		this.persist();
		this.version += 1;
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
