// 轻量 i18n：en（默认）/ zh 两个字典，扁平 key，localStorage 持久化，类型安全。
export type Params = Record<string, string | number>;
export type Message = string | ((p: Params) => string);

const en = {
	// ---- navigation ----
	'nav.stock': 'Stock',
	'nav.search': 'Search',
	'nav.gamut': 'Gamut',
	'nav.settings': 'Settings',

	// ---- paint properties ----
	'surface.title': 'Finish',
	'surface.G': 'Gloss',
	'surface.SG': 'Semi-Gloss',
	'surface.M': 'Flat',
	'surface.ME': 'Metallic',
	'surface.C': 'Clear',
	'surface.PA': 'Mica',
	'surface.FL': 'Fluo',
	'surface.W': 'Weathering',
	'surface.U': 'Unknown',
	'medium.title': 'Medium',
	'medium.Airbrush': 'Airbrush',
	'medium.Spray': 'Spray',
	'medium.Brush': 'Brush',
	'medium.Marker': 'Marker',
	'medium.Other': 'Other',
	'base.title': 'Solvent',
	'base.lacquer': 'Lacquer',
	'base.alcohol': 'Alcohol',
	'base.enamel': 'Enamel',
	'base.water': 'Water',

	// ---- search page ----
	'search.sourcePalette': 'Palette',
	'search.sourceCamera': 'Camera',
	'search.series': 'Series',
	'search.any': 'Any',
	'search.selectAll': 'Select All',
	'search.seriesCount': (p: Params) => `${p.n} series`,
	'search.paintsCount': (p: Params) => `${p.n} paints`,
	'search.hoverBrandHint': 'Select a brand<br />to view its series',
	'search.market': 'Market',
	'search.myStock': 'My Stock',
	'search.mixTitle': 'Mixing',
	'search.mixOff': 'Off',
	'search.mix1': 'Two Colors',
	'search.mix2': 'Three Colors',
	'search.mixScopeRequired': 'Only supported for `My Stock`',
	'search.resetFilter': 'Reset',
	'search.results': (p: Params) => `${p.n} Result${p.n == 1 ? '' : 's'}`,
	'search.similarity': (p: Params) => `${p.n}% similar`,
	'search.sourcePaint': 'From paint',
	'search.selectPaint': 'Select paint',
	'search.selectPaintHint': 'Select a result on the left to view details',

	// ---- stock page ----
	'stock.back': 'Back',
	'stock.brands': 'Brands',
	'stock.selectPaintHint': 'Select a paint on the left to view details',
	'stock.addToStock': 'Add to Stock',
	'stock.removeFromStock': 'Remove from Stock',
	'stock.brandStats': (p: Params) =>
		`${p.series} series · ${p.paints} paint${p.paints == 1 ? '' : 's'}`,
	'stock.directEquiv': 'Direct Equivalents',
	'stock.noDirectEquiv': 'No same-name equivalent found',
	'stock.similarColors': 'Similar Colors',
	'stock.noSimilar': 'No similar paint found',
	'stock.similarity': (p: Params) => `${p.n}% similar`,
	'stock.mixFromStock': 'Mix from stock',
	'stock.reportIssue': 'Report issue',
	'stock.viewAllSimilar': 'View all',
	'stock.searchPlaceholder': 'Search by code or name',
	'stock.sortCode': 'Code',
	'stock.sortHue': 'Hue',
	'stock.sortSat': 'Sat',
	'stock.sortLight': 'Lit',
	'stock.sortStock': 'Stock',
	'stock.sortTitle': 'Sort',
	'stock.searchTitle': 'Search',
	'stock.closeSearch': 'Close search',
	'stock.noResults': 'No matching paints',
	'stock.filterTitle': 'Filter',
	'stock.filterClear': 'Clear',
	'stock.surfaceTitle': 'Finish',
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
	'gamut.clickAddHint': 'Click <span class="font-bold">Add</span> to start',

	// ---- about page ----
	'about.privacyTitle': 'Privacy & Security Commitment',
	'about.zeroDataTitle': 'Zero Personal Data Collection',
	'about.zeroDataDesc':
		'We <strong class="font-bold">do not collect, upload, or store</strong> any of your personal data, stock configuration, or search history. Your privacy is absolutely safe.',
	'about.localTitle': '100% Local Offline',
	'about.localDesc':
		'The core color-mixing algorithm and paint lookup run entirely locally in your browser (WebAssembly) — no network connection required.',
	'about.disclaimerTitle': 'Color Mixing and Painting',
	'about.disclaimer1':
		'1. All mixing formulas and color difference results are simulated based on <a class="underline" href="https://scrtwpns.com/mixbox.pdf" target="_blank" rel="noopener noreferrer">color science theory</a>; <strong class="font-bold">actual results are for reference only</strong>.',
	'about.disclaimer2':
		'2. Due to physical differences in chemical properties, pigment density, and coverage between manufacturers and series, <strong class="font-bold">always verify mixing feasibility with the paint properties published by the manufacturer</strong>.',
	'about.disclaimer3':
		'* Tip: always test-spray a small area before large-scale application to verify the actual color.',
	'about.contributionTitle': 'Data Coverage & Contribution',
	'about.contribution1':
		'Some paint brands are not included here because no <strong class="font-bold">official color chart</strong> data has been published for them.',
	'about.contribution2':
		'If you have data for these brands and are willing to share, <strong class="font-bold">contributions are welcome</strong> — reach us via GitHub / QQ at the bottom of this page.',
	'about.buyMeCoffee': 'Buy me a coffee',
	'about.checkUpdate': 'Check Update',
	'about.checking': 'Checking…',
	'about.upToDate': 'Up to date',
	'about.updateAvailable': 'Latest: v{n}',
	'about.viewUpdate': 'View Update',
	'about.checkFailed': 'Unable to check for updates',

	// ---- settings page ----
	'settings.title': 'Settings',
	'settings.lang': 'Language',
	'settings.langDesc': 'App interface language',
	'settings.displayRaw': 'Prefer metadata in original language',
	'settings.displayRawDesc':
		'Where available, paint names will be shown in their native language (and character-set).',
	'settings.theme': 'Theme',
	'settings.themeDesc': 'Follow system, light or dark',
	'settings.themeLight': 'Light',
	'settings.themeDark': 'Dark',
	'settings.themeSystem': 'System',
	'settings.about': 'About',
	'settings.legal': 'Terms of Use',

	// ---- shared components ----
	'camera.accessError': 'Cannot access the camera (permission denied or device unavailable)',
	'camera.launching': 'Launching Camera...',
	'camera.captureColor': 'capture color',
	'colorCode.copied': 'copied!',
	'colorCode.copy': 'copy to clipboard'
} satisfies Record<string, Message>;

export type MessageKey = keyof typeof en;

const zh = {
	'nav.stock': '库存',
	'nav.search': '查询',
	'nav.gamut': '色域',
	'nav.settings': '设置',

	'surface.title': '漆面',
	'surface.G': '光泽',
	'surface.SG': '半光泽',
	'surface.M': '消光',
	'surface.ME': '金属色',
	'surface.C': '透明',
	'surface.PA': '珠光',
	'surface.FL': '荧光',
	'surface.W': '旧化',
	'surface.U': '不明',
	'base.title': '溶剂',
	'medium.title': '媒介',
	'medium.Airbrush': '喷笔',
	'medium.Spray': '喷罐',
	'medium.Brush': '笔刷',
	'medium.Marker': '马克笔',
	'medium.Other': '其他',
	'base.lacquer': '硝基漆',
	'base.alcohol': '醇基水性漆',
	'base.enamel': '珐琅漆',
	'base.water': '水性漆',

	'search.sourcePalette': '调色盘',
	'search.sourceCamera': '相机',
	'search.series': '系列',
	'search.any': '不限',
	'search.selectAll': '全选',
	'search.seriesCount': (p: Params) => `${p.n} 个系列`,
	'search.paintsCount': (p: Params) => `${p.n} 款油漆`,
	'search.hoverBrandHint': '选择一个品牌<br />查看其系列',
	'search.market': '市场',
	'search.myStock': '库存',
	'search.mixTitle': '混色',
	'search.mixOff': '关闭',
	'search.mix1': '双色',
	'search.mix2': '三色',
	'search.mixScopeRequired': '仅支持在库存中混色',
	'search.resetFilter': '重置',
	'search.results': (p: Params) => `${p.n} 条结果`,
	'search.similarity': (p: Params) => `相似度 ${p.n}%`,
	'search.sourcePaint': '来自油漆',
	'search.selectPaint': '选择油漆',
	'search.selectPaintHint': '点击左侧结果查看详情',

	'stock.back': '返回',
	'stock.brands': '品牌',
	'stock.selectPaintHint': '点击左侧油漆查看详情',
	'stock.addToStock': '加入库存',
	'stock.removeFromStock': '移出库存',
	'stock.brandStats': (p: Params) => `${p.series} 个系列 · ${p.paints} 款油漆`,
	'stock.directEquiv': '官标等价',
	'stock.noDirectEquiv': '暂无官标等价的其他油漆',
	'stock.similarColors': '相近同色漆',
	'stock.noSimilar': '暂无相近的其他油漆',
	'stock.similarity': (p: Params) => `${p.n}% 相似`,
	'stock.mixFromStock': '调配',
	'stock.reportIssue': '报告问题',
	'stock.viewAllSimilar': '查看全部',
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
	'gamut.clickAddHint': '点击 <span class="font-bold">添加</span> 开始',

	'about.privacyTitle': '隐私安全承诺',
	'about.zeroDataTitle': '零个人数据收集',
	'about.zeroDataDesc':
		'我们<strong class="font-bold">不会收集、上传或存储</strong>您的任何个人数据、库存配置或检索历史。您的隐私绝对安全。',
	'about.localTitle': '100% 离线运行',
	'about.localDesc':
		'核心调色算法与漆号检索完全在您的浏览器本地（WebAssembly）编译执行，无需联网即可稳定使用。',
	'about.disclaimerTitle': '实喷与调色提示',
	'about.disclaimer1':
		'1. 本工具所有的混色配方与色差计算结果均基于<a class="underline" href="https://scrtwpns.com/mixbox.pdf" target="_blank" rel="noopener noreferrer">色彩科学理论</a>模拟得出，<strong class="font-bold">实际发色效果仅供参考</strong>。',
	'about.disclaimer2':
		'2. 由于不同厂商以及不同系列油漆的化学性质、颜料颗粒密度及遮盖力存在物理差异，<strong class="font-bold">具体混色可行性请务必结合厂商提供的涂料性质决定</strong>。',
	'about.disclaimer3': '* 提示：在大面积喷涂前，请务必进行局部试喷以验证实际发色。',
	'about.contributionTitle': '数据覆盖与贡献',
	'about.contribution1':
		'部分油漆品牌因未发布<strong class="font-bold">官方色卡</strong>数据，暂未收录于本工具中，故未在此列出。',
	'about.contribution2':
		'如果您拥有相关品牌的数据并愿意共享，<strong class="font-bold">欢迎贡献数据</strong>，可通过页面底部的 GitHub / QQ 联系我们。',
	'about.buyMeCoffee': 'Buy me a coffee',
	'about.checkUpdate': '检查更新',
	'about.checking': '正在检查…',
	'about.upToDate': '已是最新版本',
	'about.updateAvailable': '发现新版本 v{n}',
	'about.viewUpdate': '查看更新',
	'about.checkFailed': '无法检查更新',

	// ---- settings page ----
	'settings.title': '设置',
	'settings.lang': '语言',
	'settings.langDesc': '应用界面语言',
	'settings.displayRaw': '以原语言显示油漆信息',
	'settings.displayRawDesc':
		'如果可能，油漆名称将以原始语言显示（例如，GUNZE/C20-浅蓝将会显示为ライトブルー）。',
	'settings.theme': '主题',
	'settings.themeDesc': '跟随系统、浅色或深色',
	'settings.themeLight': '浅色',
	'settings.themeDark': '深色',
	'settings.themeSystem': '跟随系统',
	'settings.about': '关于',
	'settings.legal': '使用须知',

	'camera.accessError': '无法访问摄像头（未授权或设备不可用）',
	'camera.launching': '正在启动摄像头...',
	'camera.captureColor': '拍照取色',
	'colorCode.copied': '已复制!',
	'colorCode.copy': '复制到剪贴板'
} satisfies Record<MessageKey, Message>;

const ja = {
	'nav.stock': '在庫',
	'nav.search': '検索',
	'nav.gamut': '色域',
	'nav.settings': '設定',

	'surface.title': '仕上げ',
	'surface.G': '光沢',
	'surface.SG': '半光沢',
	'surface.M': 'つや消し',
	'surface.ME': 'メタリック',
	'surface.C': 'クリア',
	'surface.PA': 'パール',
	'surface.FL': '蛍光',
	'surface.W': 'ウェザリング',
	'surface.U': '不明',
	'base.title': '溶剤',
	'medium.title': 'メディウム',
	'medium.Airbrush': 'エアブラシ',
	'medium.Spray': 'スプレー',
	'medium.Brush': '筆塗り',
	'medium.Marker': 'マーカー',
	'medium.Other': 'その他',
	'base.lacquer': 'ラッカー',
	'base.alcohol': 'アルコール',
	'base.enamel': 'エナメル',
	'base.water': '水性',

	'search.sourcePalette': 'パレット',
	'search.sourceCamera': 'カメラ',
	'search.series': 'シリーズ',
	'search.any': 'すべて',
	'search.selectAll': '全選択',
	'search.seriesCount': (p: Params) => `${p.n} シリーズ`,
	'search.paintsCount': (p: Params) => `${p.n} 色`,
	'search.hoverBrandHint': 'ブランドを選択<br />シリーズを表示',
	'search.market': '全塗料',
	'search.myStock': 'マイ在庫',
	'search.mixTitle': '混色',
	'search.mixOff': 'オフ',
	'search.mix1': '2色',
	'search.mix2': '3色',
	'search.mixScopeRequired': 'マイ在庫でのみ対応',
	'search.resetFilter': 'リセット',
	'search.results': (p: Params) => `${p.n} 件`,
	'search.similarity': (p: Params) => `類似度 ${p.n}%`,
	'search.sourcePaint': '塗料から',
	'search.selectPaint': '塗料を選択',
	'search.selectPaintHint': '左の結果をクリックして詳細を表示',

	'stock.back': '戻る',
	'stock.brands': 'ブランド',
	'stock.selectPaintHint': '左の塗料をクリックして詳細を表示',
	'stock.addToStock': '在庫に追加',
	'stock.removeFromStock': '在庫から削除',
	'stock.brandStats': (p: Params) => `${p.series} シリーズ · ${p.paints} 色`,
	'stock.directEquiv': '公式同等品',
	'stock.noDirectEquiv': '同名の同等品なし',
	'stock.similarColors': '類似色',
	'stock.noSimilar': '類似した塗料なし',
	'stock.similarity': (p: Params) => `類似度 ${p.n}%`,
	'stock.mixFromStock': '調色',
	'stock.reportIssue': '問題を報告',
	'stock.viewAllSimilar': 'すべて表示',
	'stock.searchPlaceholder': '番号または名前で検索',
	'stock.sortCode': '番号',
	'stock.sortHue': '色相',
	'stock.sortSat': '彩度',
	'stock.sortLight': '明度',
	'stock.sortStock': '在庫',
	'stock.sortTitle': '並び替え',
	'stock.searchTitle': '検索',
	'stock.closeSearch': '検索を閉じる',
	'stock.noResults': '一致する塗料なし',
	'stock.filterTitle': 'フィルター',
	'stock.filterClear': 'クリア',
	'stock.surfaceTitle': '仕上げ',
	'stock.baseTitle': '溶剤',

	'gamut.change': '変更',
	'gamut.searchPlaceholder': 'ブランド / 番号 / 名前を検索...',
	'gamut.myStock': 'マイ在庫',
	'gamut.stockCount': (p: Params) => `在庫 ${p.n} 色`,
	'gamut.color': '色',
	'gamut.paint': '塗料',
	'gamut.add': '追加',
	'gamut.clipping': 'クリッピング',
	'gamut.sources': '基準色',
	'gamut.colorsInGamut': (p: Params) => `色域内 ${p.n} 色`,
	'gamut.noSources': 'まだ基準色がありません',
	'gamut.clickAddHint': '<span class="font-bold">追加</span> をクリックして開始',

	'about.privacyTitle': 'プライバシーとセキュリティ',
	'about.zeroDataTitle': '個人データ収集ゼロ',
	'about.zeroDataDesc':
		'当ツールはお客様の個人データ、在庫設定、検索履歴を<strong class="font-bold">収集・アップロード・保存しません</strong>。プライバシーは完全に保護されます。',
	'about.localTitle': '100% ローカルオフライン',
	'about.localDesc':
		'色合わせアルゴリズムと塗料検索はすべてブラウザ内（WebAssembly）で動作します。ネットワーク接続は不要です。',
	'about.disclaimerTitle': '調色と塗装について',
	'about.disclaimer1':
		'1. すべての調色レシピと色差結果は<a class="underline" href="https://scrtwpns.com/mixbox.pdf" target="_blank" rel="noopener noreferrer">色彩科学理論</a>に基づくシミュレーションです。<strong class="font-bold">実際の発色は参考値としてお考えください</strong>。',
	'about.disclaimer2':
		'2. メーカーやシリーズごとに化学的性質、顔料密度、隠蔽力が異なるため、<strong class="font-bold">混色の可否はメーカー公表の塗料特性で必ずご確認ください</strong>。',
	'about.disclaimer3':
		'* ヒント：広範囲に塗装する前に、必ず小さな面積で試し吹きして実際の発色を確認してください。',
	'about.contributionTitle': 'データカバレッジと貢献',
	'about.contribution1':
		'一部の塗料ブランドは<strong class="font-bold">公式カラーチャート</strong>が公開されていないため未収録です。',
	'about.contribution2':
		'データをお持ちで共有いただける方は、<strong class="font-bold">貢献大歓迎</strong>です。ページ下部の GitHub / QQ からご連絡ください。',
	'about.buyMeCoffee': 'Buy me a coffee',
	'about.checkUpdate': 'アップデートを確認',
	'about.checking': '確認中...',
	'about.upToDate': '最新版です',
	'about.updateAvailable': '最新版: v{n}',
	'about.viewUpdate': '更新を見る',
	'about.checkFailed': 'アップデートを確認できません',

	// ---- settings page ----
	'settings.title': '設定',
	'settings.lang': '言語',
	'settings.langDesc': 'アプリの表示言語',
	'settings.displayRaw': '塗料情報を原言語で表示',
	'settings.displayRawDesc': '可能な場合、塗料名は元の言語（原表記）で表示されます。',
	'settings.theme': 'テーマ',
	'settings.themeDesc': 'システム / ライト / ダーク',
	'settings.themeLight': 'ライト',
	'settings.themeDark': 'ダーク',
	'settings.themeSystem': 'システム',
	'settings.about': 'について',
	'settings.legal': 'ご利用にあたって',

	'camera.accessError': 'カメラにアクセスできません（権限がないか、デバイスが利用できません）',
	'camera.launching': 'カメラ起動中...',
	'camera.captureColor': '色をキャプチャ',
	'colorCode.copied': 'コピーしました!',
	'colorCode.copy': 'クリップボードにコピー'
} satisfies Record<MessageKey, Message>;

export const messages = { en, zh, ja };

export type Locale = keyof typeof messages;

const STORAGE_KEY = 'paintbox:locale';

const detect = (): Locale => {
	if (typeof localStorage !== 'undefined') {
		const saved = localStorage.getItem(STORAGE_KEY);
		if (saved === 'en' || saved === 'zh' || saved === 'ja') return saved;
	}
	if (typeof navigator !== 'undefined') {
		const lang = navigator.language.toLowerCase();
		if (lang.startsWith('zh')) return 'zh';
		if (lang.startsWith('ja')) return 'ja';
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
