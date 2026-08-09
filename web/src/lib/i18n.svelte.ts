// 轻量 i18n：en（默认）/ zh 两个字典，扁平 key，localStorage 持久化，类型安全。
export type Params = Record<string, string | number>;
export type Message = string | ((p: Params) => string);

const en = {
	// ---- navigation ----
	'nav.stock': 'Stock',
	'nav.search': 'Search',
	'nav.gamut': 'Gamut',
	'nav.about': 'About',

	// ---- search page ----
	'search.sourcePalette': 'Palette',
	'search.sourceCamera': 'Camera',
	'search.series': 'Series:',
	'search.any': 'Any',
	'search.cancelAll': 'Reset',
	'search.selectAll': 'Select All',
	'search.seriesCount': (p: Params) => `${p.n} series`,
	'search.paintsCount': (p: Params) => `${p.n} paints`,
	'search.hoverBrandHint': 'Select a brand<br />to view its series',
	'search.surfaceTooltip': 'surface type',
	'search.surfaceTitle': 'Surface',
	'search.surface.G': 'Gloss',
	'search.surface.SG': 'Semi-Gloss',
	'search.surface.M': 'Flat',
	'search.surface.ME': 'Metallic',
	'search.surface.C': 'Clear',
	'search.surface.PA': 'Mica',
	'search.surface.FL': 'Fluorescence',
	'search.surface.W': 'Weathering',
	'search.baseTooltip': 'solvent base type',
	'search.baseTitle': 'Base',
	'search.base.0': 'Lacquer',
	'search.base.1': 'Alcohol',
	'search.base.2': 'Enamel',
	'search.base.3': 'Water',
	'search.scopeTooltip': 'search scope',
	'search.market': 'Market',
	'search.myStock': 'My Stock',
	'search.mixTooltip': 'mixing',
	'search.mixOff': 'Mix Off',
	'search.mix1': 'Mix-1',
	'search.mix2': 'Mix-2',
	'search.mixScopeRequired': 'Mixing requires search scope: My Stock',
	'search.resetFilter': 'Reset Filter',
	'search.results': (p: Params) => `${p.n} Result${p.n == 1 ? '' : 's'}`,
	'search.similarity': (p: Params) => `${p.n}% similar`,

	// ---- stock page ----
	'stock.back': 'Back',
	'stock.brands': 'Brands',
	'stock.addToStock': 'Add to Stock',
	'stock.removeFromStock': 'Remove from Stock',
	'stock.brandStats': (p: Params) =>
		`${p.series} series · ${p.paints} paint${p.paints == 1 ? '' : 's'}`,
	'stock.directEquiv': 'Direct Equivalents',
	'stock.noDirectEquiv': 'No same-name equivalent found',
	'stock.similarColors': 'Similar Colors',
	'stock.noSimilar': 'No similar paint found',
	'stock.similarity': (p: Params) => `${p.n}% similar`,
	'stock.searchPlaceholder': 'Search by code or name',
	'stock.sortCode': 'Code',
	'stock.sortHue': 'Hue',
	'stock.sortSat': 'Saturation',
	'stock.sortLight': 'Lightness',
	'stock.sortStock': 'In stock first',
	'stock.sortTitle': 'Sort',
	'stock.searchTitle': 'Search',
	'stock.closeSearch': 'Close search',
	'stock.noResults': 'No matching paints',
	'stock.filterTitle': 'Filter',
	'stock.filterClear': 'Clear',
	'stock.surfaceTitle': 'Surface',
	'stock.baseTitle': 'Base',

	// ---- gamut page ----
	'gamut.change': 'Change',
	'gamut.searchPlaceholder': 'search brand / code / name...',
	'gamut.myStock': 'My Stock',
	'gamut.stockCount': (p: Params) => `${p.n} paint${p.n == 1 ? '' : 's'}`,
	'gamut.color': 'Color',
	'gamut.paint': 'Paint',
	'gamut.add': 'Add',
	'gamut.clipping': 'Clipping',
	'gamut.sources': 'Source Colors',
	'gamut.colorsInGamut': (p: Params) => `${p.n} color${p.n == 1 ? '' : 's'} in gamut`,
	'gamut.noSources': 'No source colors yet',
	'gamut.clickAddHint': 'Click <span class="font-medium">Add</span> to start',
	'gamut.drag': 'Drag to reorder',
	'gamut.delete': 'Delete',
	'gamut.hide': 'Hide from gamut',
	'gamut.show': 'Show in gamut',

	// ---- about page ----
	'about.switchToZh': 'Switch to 中文',
	'about.switchToEn': 'Switch to English',
	'about.privacyTitle': 'Privacy & Security',
	'about.zeroDataTitle': 'Zero Personal Data Collection',
	'about.zeroDataDesc':
		'We <strong class="font-medium text-gray-300">do not collect, upload, or store</strong> any of your personal data, stock configuration, or search history. Your privacy is absolutely safe.',
	'about.localTitle': '100% Local Offline',
	'about.localDesc':
		'The core color-mixing algorithm and paint lookup run entirely locally in your browser (WebAssembly) — no network connection required.',
	'about.disclaimerTitle': 'Mixing Algorithm Disclaimer',
	'about.disclaimer1':
		'1. All mixing formulas and color difference results are simulated based on <a class="underline" href="https://scrtwpns.com/mixbox.pdf" target="_blank" rel="noopener noreferrer">color science theory</a>; <strong class="font-medium text-gray-300">actual results are for reference only</strong>.',
	'about.disclaimer2':
		'2. Due to physical differences in chemical properties, pigment density, and coverage between manufacturers and series, <strong class="font-medium text-gray-300">always verify mixing feasibility with the paint properties published by the manufacturer</strong>.',
	'about.disclaimer3':
		'* Tip: always test-spray a small area before large-scale application to verify the actual color.',
	'about.contributionTitle': 'Data Coverage & Contribution',
	'about.contribution1':
		'Some paint brands are not included here because no <strong class="font-medium text-gray-300">official color chart</strong> data has been published for them.',
	'about.contribution2':
		'If you have data for these brands and are willing to share, <strong class="font-medium text-gray-300">contributions are welcome</strong> — reach us via GitHub / QQ at the bottom of this page.',
	'about.buyMeCoffee': 'Buy me a coffee',
	'about.checkUpdate': 'Check Update',
	'about.checking': 'Checking…',
	'about.upToDate': 'Up to date',
	'about.updateAvailable': 'Latest: v{n}',
	'about.viewUpdate': 'View Update',
	'about.checkFailed': 'Unable to check for updates',

	// ---- shared components ----
	'camera.accessError': 'Cannot access the camera (permission denied or device unavailable)',
	'camera.launching': 'Launching Camera...',
	'camera.captureColor': 'capture color',
	'camera.pick': 'Pick',
	'colorCode.copied': 'copied!',
	'colorCode.copy': 'copy to clipboard'
} satisfies Record<string, Message>;

export type MessageKey = keyof typeof en;

const zh = {
	'nav.stock': '库存',
	'nav.search': '查询',
	'nav.gamut': '色域',
	'nav.about': '关于',

	'search.sourcePalette': '调色盘',
	'search.sourceCamera': '相机',
	'search.series': '系列:',
	'search.any': '不限',
	'search.cancelAll': '重置',
	'search.selectAll': '全选',
	'search.seriesCount': (p: Params) => `${p.n} 个系列`,
	'search.paintsCount': (p: Params) => `${p.n} 款油漆`,
	'search.hoverBrandHint': '选择一个品牌<br />查看其系列',
	'search.surfaceTooltip': '漆面类型',
	'search.surfaceTitle': '漆面: 不限',
	'search.surface.G': '光泽',
	'search.surface.SG': '半光泽',
	'search.surface.M': '消光',
	'search.surface.ME': '金属色',
	'search.surface.C': '透明',
	'search.surface.PA': '珠光',
	'search.surface.FL': '荧光',
	'search.surface.W': '旧化',
	'search.baseTooltip': '溶剂类型',
	'search.baseTitle': '溶剂: 不限',
	'search.base.0': '硝基漆',
	'search.base.1': '醇基水性漆',
	'search.base.2': '珐琅漆',
	'search.base.3': '水性漆',
	'search.scopeTooltip': '搜索范围',
	'search.market': '市场',
	'search.myStock': '我的库存',
	'search.mixTooltip': '混色',
	'search.mixOff': '混色-禁用',
	'search.mix1': '混色-1',
	'search.mix2': '混色-2',
	'search.mixScopeRequired': '混色需将搜索范围设为「我的库存」',
	'search.resetFilter': '重置筛选',
	'search.results': (p: Params) => `${p.n} 条结果`,
	'search.similarity': (p: Params) => `相似度 ${p.n}%`,

	'stock.back': '返回',
	'stock.brands': '品牌',
	'stock.addToStock': '加入库存',
	'stock.removeFromStock': '移出库存',
	'stock.brandStats': (p: Params) => `${p.series} 个系列 · ${p.paints} 款油漆`,
	'stock.directEquiv': '官标等价',
	'stock.noDirectEquiv': '暂无官标等价的其他油漆',
	'stock.similarColors': '相近同色漆',
	'stock.noSimilar': '暂无相近的其他油漆',
	'stock.similarity': (p: Params) => `${p.n}% 相似`,
	'stock.searchPlaceholder': '按色号或名称搜索',
	'stock.sortCode': '色号',
	'stock.sortHue': '色相',
	'stock.sortSat': '饱和度',
	'stock.sortLight': '明度',
	'stock.sortStock': '库存',
	'stock.sortTitle': '排序',
	'stock.searchTitle': '搜索',
	'stock.closeSearch': '关闭搜索',
	'stock.noResults': '未找到匹配的油漆',
	'stock.filterTitle': '筛选',
	'stock.filterClear': '清除',
	'stock.surfaceTitle': '漆面',
	'stock.baseTitle': '溶剂',

	'gamut.change': '更换',
	'gamut.searchPlaceholder': '搜索品牌 / 编号 / 名称...',
	'gamut.myStock': '我的库存',
	'gamut.stockCount': (p: Params) => `库存 ${p.n} 款`,
	'gamut.color': '颜色',
	'gamut.paint': '油漆',
	'gamut.add': '添加',
	'gamut.clipping': '裁剪',
	'gamut.sources': '基础色',
	'gamut.colorsInGamut': (p: Params) => `色域内 ${p.n} 个基础色`,
	'gamut.noSources': '还没有基础色',
	'gamut.clickAddHint': '点击 <span class="font-medium">添加</span> 开始',
	'gamut.drag': '拖动排序',
	'gamut.delete': '删除',
	'gamut.hide': '从色域中隐藏',
	'gamut.show': '在色域中显示',

	'about.switchToZh': '切换到中文',
	'about.switchToEn': '切换到英文',
	'about.privacyTitle': '隐私与安全声明',
	'about.zeroDataTitle': '零个人数据收集',
	'about.zeroDataDesc':
		'我们<strong class="font-medium text-gray-300">不会收集、上传或存储</strong>您的任何个人数据、库存配置或检索历史。您的隐私绝对安全。',
	'about.localTitle': '100% 本地离线运行',
	'about.localDesc':
		'核心调色算法与漆号检索完全在您的浏览器本地（WebAssembly）编译执行，无需联网即可稳定使用。',
	'about.disclaimerTitle': '混色算法免责声明',
	'about.disclaimer1':
		'1. 本工具所有的混色配方与色差计算结果均基于<a class="underline" href="https://scrtwpns.com/mixbox.pdf" target="_blank" rel="noopener noreferrer">色彩科学理论</a>模拟得出，<strong class="font-medium text-gray-300">实际发色效果仅供参考</strong>。',
	'about.disclaimer2':
		'2. 由于不同厂商以及不同系列油漆的化学性质、颜料颗粒密度及遮盖力存在物理差异，<strong class="font-medium text-gray-300">具体混色可行性请务必结合厂商提供的涂料性质决定</strong>。',
	'about.disclaimer3': '* 提示：在大面积喷涂前，请务必进行局部试喷以验证实际发色。',
	'about.contributionTitle': '数据覆盖与贡献',
	'about.contribution1':
		'部分油漆品牌因未发布<strong class="font-medium text-gray-300">官方色卡</strong>数据，暂未收录于本工具中，故未在此列出。',
	'about.contribution2':
		'如果您拥有相关品牌的数据并愿意共享，<strong class="font-medium text-gray-300">欢迎贡献数据</strong>，可通过页面底部的 GitHub / QQ 联系我们。',
	'about.buyMeCoffee': 'Buy me a coffee',
	'about.checkUpdate': '检查更新',
	'about.checking': '正在检查…',
	'about.upToDate': '已是最新版本',
	'about.updateAvailable': '发现新版本 v{n}',
	'about.viewUpdate': '查看更新',
	'about.checkFailed': '无法检查更新',

	'camera.accessError': '无法访问摄像头（未授权或设备不可用）',
	'camera.launching': '正在启动摄像头...',
	'camera.captureColor': '拍照取色',
	'camera.pick': '取色',
	'colorCode.copied': '已复制!',
	'colorCode.copy': '复制到剪贴板'
} satisfies Record<MessageKey, Message>;

export const messages = { en, zh };

export type Locale = keyof typeof messages;

const STORAGE_KEY = 'paintbox:locale';

const detect = (): Locale => {
	if (typeof localStorage !== 'undefined') {
		const saved = localStorage.getItem(STORAGE_KEY);
		if (saved === 'en' || saved === 'zh') return saved;
	}
	if (typeof navigator !== 'undefined' && navigator.language.toLowerCase().startsWith('zh')) {
		return 'zh';
	}
	return 'en';
};

class I18nStore {
	locale = $state<Locale>(detect());

	/** 设置语言并持久化，同步 <html lang> */
	set(l: Locale) {
		this.locale = l;
		try {
			localStorage.setItem(STORAGE_KEY, l);
		} catch {
			// 存储不可用（如隐私模式）时仅保留内存态
		}
		if (typeof document !== 'undefined') document.documentElement.lang = l;
	}

	toggle() {
		this.set(this.locale === 'en' ? 'zh' : 'en');
	}

	/** 按 key 取翻译；支持 {name} 插值，函数型消息用于复数等场景 */
	t(key: MessageKey, params?: Params): string {
		const msg = messages[this.locale][key] ?? key;
		if (typeof msg === 'function') return msg(params ?? {});
		if (params === undefined) return msg;
		return msg.replace(/\{(\w+)\}/g, (_, k: string) => String(params[k] ?? ''));
	}
}

export const i18n = new I18nStore();

// 便捷函数，供模板直接调用；内部读取 i18n.locale，保持响应式
export const t = (key: MessageKey, params?: Params) => i18n.t(key, params);
export const toggleLocale = () => i18n.toggle();

// 首次加载即同步 <html lang>（纯客户端应用）
if (typeof document !== 'undefined') document.documentElement.lang = i18n.locale;
