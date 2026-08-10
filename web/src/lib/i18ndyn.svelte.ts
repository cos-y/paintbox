import { i18n } from './i18n.svelte';

/**
 * 油漆色名的动态本地化字典：{ brand: { code: 色名 } }
 * 按当前语言只加载对应的 json（zh.json / en.json，两份均覆盖全部品牌），
 * 切换语言时动态加载另一份；raw.json 不再加载。
 * 回退链：目标语言字典 → code。
 */
type PaintNames = Record<string, Record<string, string>>;

const FILES = {
	zh: '/paints/zh.json',
	en: '/paints/en.json'
} as const;

// 已加载字典缓存（切回旧语言无需重新 fetch）
const cache = new Map<string, PaintNames>();
// 当前生效字典，$state 保证模板响应式
let dict: PaintNames | null = $state(null);

const load = async (locale: string): Promise<PaintNames> => {
	const hit = cache.get(locale);
	if (hit) return hit;
	const data = (await fetch(FILES[locale as keyof typeof FILES] ?? FILES.en).then((r) => r.json())) as PaintNames;
	cache.set(locale, data);
	return data;
};

/** layout load 中调用：按当前语言预载字典（用 SvelteKit fetch，prerender 期可用） */
export const loadPaintNames = async (fetchFn: typeof fetch) => {
	const locale = i18n.locale;
	const data = (await fetchFn(FILES[locale as keyof typeof FILES] ?? FILES.en).then((r) => r.json())) as PaintNames;
	cache.set(locale, data);
	dict = data;
};

// 客户端跟踪语言切换：locale 变化时加载对应字典，加载完成赋值 dict 触发重渲染
if (typeof window !== 'undefined') {
	$effect.root(() => {
		$effect(() => {
			const locale = i18n.locale;
			// 若 loadPaintNames 已预载则立即命中缓存
			void load(locale).then((d) => {
				if (i18n.locale === locale) dict = d;
			});
		});
	});
}

/** 按当前语言查色名。serie 为兼容参数（字典按 brand+code 组织，code 在品牌内唯一）。 */
export const paintName = (brand: string, _serie: string, code: string): string =>
	dict?.[brand]?.[code] ?? code;

/** 便捷：直接传 paint/portion 对象（portion 无 serie 字段，类型放宽） */
export const paintDesc = (p: { brand: string; code: string }): string =>
	paintName(p.brand, '', p.code);
