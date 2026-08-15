// 等价表：equivs.bin 是 msgpack [n_pairs, dict_sources, src_idx[], dst_idx[], source_id[]]
// 单向（仅源声明方向，每条带声明来源）。前端在 layout load 时解析一次，
// 解析时补充反向（反向继承 source），详情页按 index O(1) 查询。不做传递闭包。
import { decode } from '@msgpack/msgpack';

export interface EquivRef {
	idx: number;
	source: string; // 等价声明来源（sources registry id）
}

let equivMap: Map<number, EquivRef[]> | null = null;

export const initEquivs = (buf: ArrayBuffer): void => {
	const [n, dict, srcs, dsts, sids] = decode(new Uint8Array(buf)) as [
		number,
		string[],
		number[],
		number[],
		number[]
	];
	const m = new Map<number, EquivRef[]>();
	const add = (a: number, b: number, source: string) => {
		let list = m.get(a);
		if (!list) m.set(a, (list = []));
		list.push({ idx: b, source });
	};
	// 同一无向等价对（含双向声明）按 (min,max,source) 去重：C1 声明 H1 与
	// H1 声明 C1 是同一条等价，只保留一次；不同 source 的声明各自保留
	const seen = new Set<string>();
	for (let k = 0; k < n; k++) {
		const source = dict[sids[k]];
		const [a, b] = srcs[k] < dsts[k] ? [srcs[k], dsts[k]] : [dsts[k], srcs[k]];
		const key = `${a},${b},${source}`;
		if (seen.has(key)) continue;
		seen.add(key);
		add(a, b, source);
		add(b, a, source); // 反向补充（同一声明，source 继承）
	}
	equivMap = m;
};

/** 与指定油漆（内部下标）直接等价的其它油漆下标列表（双向，不含传递） */
export const findEquivIndices = (index: number): number[] =>
	equivMap?.get(index)?.map((e) => e.idx) ?? [];

/** 直接等价（双向，带声明来源） */
export const findEquivs = (index: number): EquivRef[] => equivMap?.get(index) ?? [];
