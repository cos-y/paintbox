import { list_paints, search } from '../wasm-pkg/paintbox_wasm';

export interface PaintInfo {
	brand: string;
	code: string;
	desc: string;
	serie: string;
	serie_code: string;
	rgb: number;
}

export const paintId = (paint: PaintInfo) => `${paint.brand}:${paint.code}`;

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

export const searchNearest = (rgb: number, maxMix: number, limit: number): SearchResult[] => {
	return (search(rgb, maxMix, limit) as SearchResult[]) ?? [];
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
