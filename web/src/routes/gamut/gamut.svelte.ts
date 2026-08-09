const STORAGE_KEY = 'paintbox:gamut';

export interface SerializedSource {
	id: string;
	type: 'color' | 'paint' | 'stock';
	/** color card: 6-digit hex as number */
	rgb?: number;
	/** paint card: brand:code id */
	paintId?: string;
	/** 暂时从色域中排除（不移除卡片） */
	hidden?: boolean;
}

export interface GamutData {
	sources: SerializedSource[];
	clipL: [number, number];
	clipA: [number, number];
	clipB: [number, number];
	nextId: number;
	/** localStorage 里是否有已保存的记录（区分首次使用 vs 用户保存过但清空了 sources） */
	persisted: boolean;
}

export function loadGamut(): GamutData {
	if (typeof localStorage === 'undefined')
		return {
			sources: [],
			clipL: [0, 16],
			clipA: [-12, 14],
			clipB: [-15, 12],
			nextId: 0,
			persisted: false
		};
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (raw) return { ...JSON.parse(raw), persisted: true };
	} catch {}
	return {
		sources: [],
		clipL: [0, 16],
		clipA: [-12, 14],
		clipB: [-15, 12],
		nextId: 0,
		persisted: false
	};
}

export function saveGamut(data: GamutData) {
	localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
}
