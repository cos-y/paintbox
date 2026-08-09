import {
	list_paints,
	search,
	color_diff,
	find_direct_equivalences
} from '../wasm-pkg/paintbox_wasm';

export interface PaintInfo {
	index: number;
	brand: string;
	code: string;
	desc: string;
	serie: string;
	serie_code: string;
	rgb: number;
	base: number;
	/** 漆面类型，SurfaceType 单 bit 值（1=G, 2=SG, 4=M…） */
	prop: number;
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

export const paintId = (paint: { brand: string; code: string }) => `${paint.brand}:${paint.code}`;

export const rgbToHex = (rgb: number) => `#${rgb.toString(16).padStart(6, '0')}`;

export const floatRgbToCss = ([r, g, b]: [number, number, number]) =>
	`rgb(${Math.round(r * 255)} ${Math.round(g * 255)} ${Math.round(b * 255)})`;

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

export interface FilterOptions {
	series?: string[][];
	all?: number[];
	/** 漆面类型 bitmask；缺省/0 = 不限制 */
	surfaces?: number;
	bases?: number[];
	mix?: number;
	limit?: number;
}

export const colorDiff = (rgbA: number, rgbB: number): number => color_diff(rgbA, rgbB);

// 直接等价：数据来源里的品牌对照表（例如Gunze H9 <-> Gunze C9），名字/型号对应但颜色不一定相近；
// 索引在wasm init时就建好了，这里只是O(1)查询
export const findDirectEquivalences = (index: number): PaintInfo[] =>
	(find_direct_equivalences(index) as PaintInfo[]) ?? [];

export const searchNearest = (rgb: number, opts: FilterOptions = {}): SearchResult[] => {
	return (search(rgb, opts) as SearchResult[]) ?? [];
};

export interface PaintCatalog {
	[k: string]: {
		[k: string]: PaintInfo[];
	};
}

// ---- 全局数据：模块级单例，页面间导航不重新计算（数据源在 wasm 里一次性加载，不可变）----

let paints = $state<PaintInfo[] | null>(null);

export const listPaints = (): PaintInfo[] => {
	if (paints === null) {
		paints = (list_paints() as PaintInfo[]) ?? [];
	}
	return paints;
};

let catalog = $state<PaintCatalog | null>(null);

export const getCatalog = (paintsArg: PaintInfo[]): PaintCatalog => {
	if (catalog === null) {
		const c: PaintCatalog = {};
		for (const paint of paintsArg) {
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
