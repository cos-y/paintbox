import { list_paints, search, color_diff, find_direct_equivalences } from '../wasm-pkg/paintbox_wasm';

export interface PaintInfo {
	index: number;
	brand: string;
	code: string;
	desc: string;
	serie: string;
	serie_code: string;
	rgb: number;
}

export const paintId = (paint: { brand: string; code: string }) => `${paint.brand}:${paint.code}`;

export const rgbToHex = (rgb: number) => `#${rgb.toString(16).padStart(6, '0')}`;

export const floatRgbToCss = ([r, g, b]: [number, number, number]) =>
	`rgb(${Math.round(r * 255)} ${Math.round(g * 255)} ${Math.round(b * 255)})`;

export const listPaints = (): PaintInfo[] => {
	return (list_paints() as PaintInfo[]) ?? [];
};

export interface SearchResultPortion {
	t: number;
	brand: string;
	code: string;
	desc: string;
	rgb: [number, number, number];
}

export interface SearchResult {
	delta_e: number;
	rgb: [number, number, number];
	portions: SearchResultPortion[];
}

export interface SerieKey {
	brand: string;
	serie: string;
}

export interface FilterOptions {
	series: SerieKey[];
	owned?: number[];
}

export const colorDiff = (rgbA: number, rgbB: number): number => color_diff(rgbA, rgbB);

// 直接等价：数据来源里的品牌对照表（例如Gunze H9 <-> Gunze C9），名字/型号对应但颜色不一定相近；
// 索引在wasm init时就建好了，这里只是O(1)查询
export const findDirectEquivalences = (index: number): PaintInfo[] =>
	(find_direct_equivalences(index) as PaintInfo[]) ?? [];

export const searchNearest = (
	rgb: number,
	maxMix: number,
	limit: number,
	filter: FilterOptions = { series: [] }
): SearchResult[] => {
	return (search(rgb, maxMix, limit, filter) as SearchResult[]) ?? [];
};

export interface SerieGroup {
	serie: string;
	paints: PaintInfo[];
}

export interface BrandGroup {
	brand: string;
	series: SerieGroup[];
}

export const groupPaints = (paints: PaintInfo[]): BrandGroup[] => {
	const brandMap = new Map<string, Map<string, PaintInfo[]>>();

	for (const paint of paints) {
		let serieMap = brandMap.get(paint.brand);
		if (!serieMap) {
			serieMap = new Map();
			brandMap.set(paint.brand, serieMap);
		}
		let list = serieMap.get(paint.serie);
		if (!list) {
			list = [];
			serieMap.set(paint.serie, list);
		}
		list.push(paint);
	}

	return [...brandMap.entries()]
		.sort(([a], [b]) => a.localeCompare(b))
		.map(([brand, serieMap]) => ({
			brand,
			series: [...serieMap.entries()]
				.sort(([a], [b]) => a.localeCompare(b))
				.map(([serie, paints]) => ({ serie, paints }))
		}));
};
