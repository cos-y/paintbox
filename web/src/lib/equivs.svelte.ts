// 官标等价：equivalences.csv 是 wasm majors 内部下标对（双向，8122 对）。
// 前端在 layout load 时解析一次，详情页按 index O(1) 查询。
let equivMap: Map<number, number[]> | null = null;

export const initEquivs = (csvText: string): void => {
	const m = new Map<number, number[]>();
	for (const row of csvText.split('\n')) {
		const line = row.trim();
		if (!line) continue;
		const [a, b] = line.split(',');
		const ai = Number(a);
		const bi = Number(b);
		if (Number.isNaN(ai) || Number.isNaN(bi)) continue;
		if (!m.has(ai)) m.set(ai, []);
		m.get(ai)!.push(bi);
		if (!m.has(bi)) m.set(bi, []);
		m.get(bi)!.push(ai);
	}
	equivMap = m;
};

/** 与指定油漆（内部下标）直接等价的其它油漆下标列表 */
export const findEquivIndices = (index: number): number[] => equivMap?.get(index) ?? [];
