import { i18n } from './i18n.svelte';
import { settings } from './settings.svelte';

/**
 * 油漆色名的动态本地化字典：{ brand: { code: 色名 } }
 * 按当前语言只加载对应的 json（zh.json / en.json，两份均覆盖全部品牌），
 * 切换语言时动态加载另一份。
 * raw.json 为源语言（原始发布）色名：设置里开启「源语言显示」时使用。
 * 回退链：目标语言字典/源语言字典 → code。
 */
type PaintNames = Record<string, Record<string, string>>;

const FILES: Record<string, string> = {
	zh: '/paints/zh.json',
	en: '/paints/en.json',
	ja: '/paints/ja.json',
	es: '/paints/es.json',
	raw: '/paints/raw.json'
} as const;

// 已加载字典缓存（切回旧语言无需重新 fetch）
const cache = new Map<string, PaintNames>();
// 当前生效字典，$state 保证模板响应式
let dict: PaintNames | null = $state(null);
// 源语言（raw）字典：useSourceNames 开启时预载
let rawDict: { [_: string]: PaintNames } | null = $state(null);

const loadFile = async (file: string, fetchFn: typeof fetch = fetch): Promise<any> => {
	const hit = cache.get(file);
	if (hit) return hit;
	const data = await fetchFn(file).then((r) => r.json());
	cache.set(file, data);
	return data;
};

/** layout load 中调用：按当前语言预载字典（用 SvelteKit fetch，prerender 期可用） */
export const preloadPaintNames = async (fetchFn: typeof fetch) => {
	dict = await loadFile(FILES[i18n.locale] ?? FILES.en, fetchFn);
	return dict;
};

// 客户端跟踪语言切换：locale 变化时加载对应字典，加载完成赋值 dict 触发重渲染
if (typeof window !== 'undefined') {
	$effect.root(() => {
		$effect(() => {
			const locale = i18n.locale;
			// 若 loadPaintNames 已预载则立即命中缓存
			void loadFile(FILES[locale] ?? FILES.en).then((d) => {
				if (i18n.locale === locale) dict = d;
			});
		});
		// 源语言模式：useSourceNames 开启时预载 raw 字典
		$effect(() => {
			if (!settings.displayRaw) return;
			void loadFile(FILES.raw).then((d) => {
				rawDict = d;
			});
		});
	});
}

/** 按当前语言查色名（开启源语言显示时改用 raw 字典）。serie 为兼容参数（字典按 brand+code 组织，code 在品牌内唯一）。 */
export const paintName = (brand: string, _serie: string, code: string): string => {
	// raw 字典未加载完成的瞬时窗口期先回退本地化字典，避免闪现 code
	if (settings.displayRaw && rawDict) {
		const rawLocale = rawDict[i18n.locale]?.[brand]?.[code];
		if (rawLocale) {
			return rawLocale;
		}

		const rawEn = rawDict.en?.[brand]?.[code];
		if (rawEn) {
			return rawEn;
		}

		for (const v of Object.values(rawDict)) {
			const raw = v[brand]?.[code];
			if (raw !== undefined) {
				return raw;
			}
		}
	}
	return dict?.[brand]?.[code] ?? code;
};

/** 便捷：直接传 paint/portion 对象（portion 无 serie 字段，类型放宽） */
export const paintDesc = (p: { brand: string; code: string }): string =>
	paintName(p.brand, '', p.code);
