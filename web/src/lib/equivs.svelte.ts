// 等价表：equivs.bin 是 msgpack [n_pairs, dict_sources, src_idx[], dst_idx[], source_id[]]
// 数据已在生成时双向展开并按 (src_idx, dst_idx) 去重（source 取 publishTs 最新），
// 这里纯读取：每对 (a,b) 唯一，直接按 index 建 Map。不做传递闭包。
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
	for (let k = 0; k < n; k++) {
		const a = srcs[k];
		let list = m.get(a);
		if (!list) m.set(a, (list = []));
		list.push({ idx: dsts[k], source: dict[sids[k]] });
	}
	equivMap = m;
};

/** 直接等价（双向，带声明来源） */
export const getEquivs = (index: number): EquivRef[] => equivMap?.get(index) ?? [];
