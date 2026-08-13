import { list_paints, search, color_diff } from '../wasm-pkg/paintbox_wasm';
import { paintDesc } from './i18ndyn.svelte';
import { hexToRgb, toHsl } from './utils.svelte';

export interface PaintInfo {
	index: number;
	brand: string;
	code: string;
	serie: string;
	serie_code: string;
	rgb: number;
	bases: number;
	surfaces: number;
	mediums: number;
	// js specific
	hsl: [number, number, number];
	id: string;
}

// SurfaceType 位定义，与 wasm 端 bitflags 对齐
// prettier-ignore
export const SURFACE_BITS: Record<string, number> = {
	G: 1 << 0,
	SG: 1 << 1,
	M: 1 << 2,
	ME: 1 << 3,
	C: 1 << 4,
	PA: 1 << 5,
	FL: 1 << 6,
	W: 1 << 7,
	U: 1 << 8
};

// MediumType 位定义，与 wasm 端 bitflags 对齐
// prettier-ignore
export const MEDIUM_BITS: Record<string, number> = {
	Airbrush: 1 << 0,
	Spray: 1 << 1,
	Brush: 1 << 2,
	Marker: 1 << 3,
	Other: 1 << 7
};

export const paintId = (paint: { brand: string; code: string }) => `${paint.brand}:${paint.code}`;

export const rgbToHex = (rgb: number) => `#${rgb.toString(16).padStart(6, '0')}`;

export const floatRgbToCss = ([r, g, b]: [number, number, number]) =>
	`rgb(${Math.round(r * 255)} ${Math.round(g * 255)} ${Math.round(b * 255)})`;

export interface SearchResultPortion {
	t: number;
	brand: string;
	code: string;
	rgb: [number, number, number];
}

export interface SearchResult {
	delta_e: number;
	rgb: [number, number, number];
	portions: SearchResultPortion[];
}

export interface FilterOptions {
	series?: string[][];
	all?: number[];
	/** 漆面类型 bitmask；缺省/0 = 不限制 */
	surfaces?: number;
	bases?: number;
	/** 介质类型 bitmask；缺省/0 = 不限制 */
	mediums?: number;
	mix?: number;
	limit?: number;
}

export const colorDiff = (rgbA: number, rgbB: number): number => color_diff(rgbA, rgbB);

// 直接等价：数据来源里的品牌对照表（例如Gunze H9 <-> Gunze C9），名字/型号对应但颜色不一定相近；
// 索引在wasm init时就建好了，这里只是O(1)查询
export const findDirectEquivalences = (index: number): PaintInfo[] => [];

export const searchNearest = (rgb: number, opts: FilterOptions = {}): SearchResult[] => {
	// 显式传 mediums: 0（不限制），避免 wasm 端 serde default（Airbrush）误过滤
	return (search(rgb, { mediums: 0, ...opts }) as SearchResult[]) ?? [];
};

export interface PaintCatalog {
	[k: string]: {
		[k: string]: PaintInfo[];
	};
}

// ---- 全局数据：模块级单例，页面间导航不重新计算（数据源在 wasm 里一次性加载，不可变）----

let paints: PaintInfo[];
let paintLookup: Map<string, PaintInfo>;

export const listPaints = (): PaintInfo[] => {
	if (paints === undefined) {
		paints = list_paints().map(({ rgb, ...info }: any) => {
			const [r, g, b] = hexToRgb(rgb!);
			const { h, s, l } = toHsl({ mode: 'rgb', r, g, b });
			return {
				hsl: [h ?? 0, s, l],
				id: paintId(info),
				rgb,
				...info
			} satisfies PaintInfo;
		});
	}
	return paints!;
};

export const getPaintByIndex = (index: number): PaintInfo | null => {
	const li = listPaints();
	return li[index] ?? null;
};

export const getPaintById = (id: string): PaintInfo | null => {
	if (paintLookup === undefined) {
		paintLookup = new Map();
		for (const p of listPaints()) {
			paintLookup.set(p.id, p);
		}
	}
	return paintLookup.get(id) ?? null;
};

let catalog: PaintCatalog;

export const getCatalog = (): PaintCatalog => {
	if (catalog === undefined) {
		const c: PaintCatalog = {};
		for (const paint of listPaints()) {
			let brand = c[paint.brand];
			if (brand === undefined) {
				brand = c[paint.brand] = {};
			}

			let serie = brand[paint.serie];
			if (serie === undefined) {
				serie = brand[paint.serie] = [];
			}

			serie.push(paint);
		}
		catalog = c;
	}
	return catalog;
};

// TODO: better fuzzy search
export const searchPaints = (query: string): PaintInfo[] => {
	if (!query) return [];
	const q = query.toLowerCase();
	return listPaints()
		.filter(
			(p) =>
				p.code.toLowerCase().includes(q) ||
				p.brand.toLowerCase().includes(q) ||
				paintDesc(p).toLowerCase().includes(q) ||
				`${p.brand} ${p.code}`.toLowerCase().includes(q)
		)
		.slice(0, 20);
};
