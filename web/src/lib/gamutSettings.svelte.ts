const STORAGE_KEY = 'paintbox:gamutSettings';

export interface SerializedSource {
	id: string;
	type: 'color' | 'paint' | 'stock';
	/** color card: 6-digit hex as number */
	rgb?: number;
	/** paint card: brand:code id */
	paintId?: string;
}

export interface GamutData {
	sources: SerializedSource[];
	clipL: [number, number];
	clipA: [number, number];
	clipB: [number, number];
	nextId: number;
}

export function loadGamut(): GamutData {
	if (typeof localStorage === 'undefined')
		return { sources: [], clipL: [0, 16], clipA: [-12, 14], clipB: [-15, 12], nextId: 0 };
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (raw) return JSON.parse(raw);
	} catch {}
	return { sources: [], clipL: [0, 16], clipA: [-12, 14], clipB: [-15, 12], nextId: 0 };
}

export function saveGamut(data: GamutData) {
	localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
}
