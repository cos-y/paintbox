import Fuse from 'fuse.js';
import { listPaints, SURFACE_BITS, MEDIUM_BITS, type PaintInfo } from './paints.svelte';
import { paintDesc } from './i18ndyn.svelte';
import { i18n } from './i18n.svelte';

/**
 * 漆号模糊搜索：Fuse.js + 预构建文档数组（懒构建，首次搜索时初始化）。
 * - 分词（空格/斜杠/连字符/句号），每词独立搜索后 AND 交集（"gaia 042" = gaia ∧ 042）
 * - 多字段加权：code > serie_code > brand > tags > serie > 色名 > sources
 * - surface/medium 位 → 多语言 tag 词（珠光/珍珠/pearl/パール…），可搜"珠光/金属/透明/荧光"
 * - 色名初始为当前语言；首次搜索时异步加载 en.json（英文全覆盖）+ raw.json（en/es/ja 源语言）merge
 *   （跨语言搜索：中文界面也能搜 "brown" 等英文色名）
 * Fuse 无倒排索引（全库几千条，逐条评分毫秒级），merge 直接改文档字段无需重建。
 */

// 位 → 多语言 tag 词（含常见同义词/别名）
const SURFACE_TAGS: Record<string, string[]> = {
	G: ['光泽', 'gloss', '光沢'],
	SG: ['半光', 'satin', '半光沢'],
	M: ['消光', 'matte', 'flat', 'つや消し'],
	ME: ['金属', 'metallic', 'メタリック'],
	C: ['透明', 'clear', 'glaze', 'クリア'],
	PA: ['珠光', '珍珠', 'pearl', 'パール'],
	FL: ['荧光', 'fluorescent', 'neon', '蛍光'],
	W: ['白', 'white', 'ホワイト'],
	U: ['uv', '紫外线', '紫外線']
};

const MEDIUM_TAGS: Record<string, string[]> = {
	Airbrush: ['喷笔', 'airbrush', 'エアブラシ'],
	Spray: ['喷罐', 'spray', 'スプレー'],
	Brush: ['笔涂', 'brush', '筆'],
	Marker: ['马克笔', 'marker', 'ペン'],
	Other: []
};

interface Doc {
	paint: PaintInfo;
	brand: string;
	code: string;
	serie_code: string;
	serie: string;
	tags: string[];
	sources: string[];
	names: string[];
}

let fuse: Fuse<Doc> | null = null;
let docs: Doc[] = [];
// 索引构建时的 UI 语言：切换语言后首次搜索自动重建（names 取新语言色名）
let builtLocale: string | null = null;
// 已 merge 的额外色名数据引用（语言切换重建索引时重新合并，不丢失）
let extraRaw: Record<string, Record<string, Record<string, string>>> | null = null;
let extraPromise: Promise<void> | null = null;

/** 位集合 → 词列表 */
const bitsToTags = (bits: number, bitDef: Record<string, number>, words: Record<string, string[]>): string[] => {
	const out: string[] = [];
	for (const [name, w] of Object.entries(words)) {
		if (bits & (bitDef[name] ?? 0)) out.push(...w);
	}
	return out;
};

/** 把色名合并进给定文档数组（目标语言字典 {brand:{code:name}}，按 brand 分组 O(N)） */
const mergeInto = (target: Doc[], dict: Record<string, Record<string, string>>) => {
	const byBrand = new Map<string, Doc[]>();
	for (const d of target) {
		if (!byBrand.has(d.brand)) byBrand.set(d.brand, []);
		byBrand.get(d.brand)!.push(d);
	}
	for (const [brand, codes] of Object.entries(dict)) {
		const group = byBrand.get(brand);
		if (!group) continue;
		for (const [code, name] of Object.entries(codes)) {
			const d = group.find((x) => x.code === code);
			if (d && name && !d.names.includes(name)) d.names.push(name);
		}
	}
};

/** 把 raw 色名合并进给定文档数组（raw.json {locale:{brand:{code:name}}}） */
const mergeRawInto = (target: Doc[], raw: Record<string, Record<string, Record<string, string>>>) => {
	for (const locale of Object.values(raw)) mergeInto(target, locale);
};

/**
 * 异步加载 raw.json（en/es/ja 源语言色名，非机翻）进搜索文档。
 * 幂等：只 fetch 一次；挂载时调用（whenExtraLoaded），搜索入口也兜底触发。
 * Fuse 无索引，merge 直接改文档字段即可生效。
 */
const loadExtra = () => {
	extraPromise ??= fetch('/paints/raw.json')
		.then((r) => r.json())
		.catch(() => null)
		.then((raw) => {
			extraRaw = raw as Record<string, Record<string, Record<string, string>>> | null;
			if (extraRaw) mergeRawInto(docs, extraRaw);
		});
	return extraPromise;
};

/** 等待额外色名（raw.json）加载完成：搜索控件 mount 时调用，保证首次搜索即可命中 */
export const whenExtraLoaded = (): Promise<void> => loadExtra();

const ensureIndex = () => {
	const locale = i18n.locale;
	if (fuse && builtLocale === locale) return;
	const built = listPaints().map((p) => ({
		paint: p,
		brand: p.brand,
		code: p.code,
		serie_code: p.serie_code,
		serie: p.serie,
		tags: [
			...bitsToTags(p.surfaces, SURFACE_BITS, SURFACE_TAGS),
			...bitsToTags(p.mediums, MEDIUM_BITS, MEDIUM_TAGS)
		],
		sources: p.sources ?? [],
		names: [paintDesc(p)]
	}));
	if (extraRaw) mergeRawInto(built, extraRaw);
	docs = built;
	fuse = new Fuse(docs, {
		keys: [
			{ name: 'code', weight: 4 },
			{ name: 'serie_code', weight: 3 },
			{ name: 'brand', weight: 2.5 },
			{ name: 'tags', weight: 2 },
			{ name: 'serie', weight: 1.5 },
			{ name: 'names', weight: 1.2 },
			{ name: 'sources', weight: 1 }
		],
		threshold: 0.35,
		ignoreLocation: true,
		minMatchCharLength: 1,
		ignoreFieldNorm: true,
		includeScore: true
	});
	builtLocale = locale;
};

const tokenize = (query: string): string[] =>
	query.toLowerCase().split(/[\s\-_/．。、,，.]+/).filter(Boolean);

/** 分词 + 逐词 Fuse 搜索 + AND 交集；返回 id → {doc, score}（score 为各词得分和，越小越优） */
const searchHits = (tokens: string[]): Map<string, { doc: Doc; score: number }> => {
	const acc = new Map<string, { doc: Doc; score: number }>();
	for (const t of tokens) {
		const hits = fuse!.search(t);
		if (acc.size === 0) {
			for (const h of hits) acc.set(h.item.paint.id, { doc: h.item, score: h.score ?? 1 });
		} else {
			const keep = new Set(hits.map((h) => h.item.paint.id));
			for (const id of [...acc.keys()]) {
				if (!keep.has(id)) acc.delete(id);
			}
			for (const h of hits) {
				const v = acc.get(h.item.paint.id);
				if (v) v.score += h.score ?? 1;
			}
		}
		if (acc.size === 0) break;
	}
	return acc;
};

/** 选择器式搜索：分词 + 逐词 Fuse 搜索 + AND 交集 + 得分排序，取 top 20 */
export const searchPaints = (query: string): PaintInfo[] => {
	if (!query.trim()) return [];
	loadExtra();
	ensureIndex();
	const tokens = tokenize(query);
	if (!tokens.length) return [];
	return [...searchHits(tokens).values()]
		.sort((a, b) => a.score - b.score)
		.slice(0, 20)
		.map((v) => v.doc.paint);
};

/** 浏览式搜索：全量命中 id 集合（不截断、无相关度排序），供 StockPage 做过滤/系列联动 */
export const searchPaintIds = (query: string): Set<string> => {
	if (!query.trim()) return new Set();
	loadExtra();
	ensureIndex();
	const tokens = tokenize(query);
	if (!tokens.length) return new Set();
	return new Set(searchHits(tokens).keys());
};
