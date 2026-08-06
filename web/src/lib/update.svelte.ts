import { isTauri } from '$lib/utils';

// 更新检测：
// - play 渠道：走 Google Play 应用内更新插件（plugin:app-update|check，按 versionCode 对比商店版本）
// - sideload 渠道：走 GitHub Release 的 version.json 资产（releases/latest/download，跟随 302 到实际资产）
// UpdateChecker 为抽象基类（统一状态机/错误处理），各渠道用子类继承接管检查逻辑，
// 工厂 createUpdateChecker 按 __CHANNEL__ 构造对应实例。

/** 最新版本来源（sideload 渠道）：GitHub Release 资产 version.json。
 * 资产 URL 会 302 到不带 CORS 头的 release-assets.githubusercontent.com，WebView 原生 fetch 会被跨域拦截。
 * 因此 Tauri 环境经 tauri-plugin-http 在 Rust 侧发起请求（不经 WebView，无 CORS、自动跟随重定向）；
 * 浏览器环境退回原生 fetch（无法绕过 CORS，该场景仅 Tauri App 使用）。 */
const RELEASE_URLS = ['https://github.com/cos-y/paintbox/releases/latest/download/version.json'];

interface PlayUpdateInfo {
	available: boolean;
	inProgress?: boolean;
	versionCode?: number;
}

/**
 * 各渠道的更新/Changelog 页面（channel 由打包时 CHANNEL 环境变量注入；无则视为 sideload = GitHub 渠道）。
 * 只引导用户「查看更新内容」，不提供直接下载入口。
 */
const CHANNELS: Record<string, string> = {
	play: 'https://play.google.com/store/apps/details?id=com.cosy.paintbox',
	sideload: 'https://github.com/cos-y/paintbox/releases/latest'
	// 未来上架国内商店时加：huawei: 'https://appgallery.huawei.com/app/...', ...
};

/** 语义化版本比较：a > b 返回 >0，a == b 返回 0 */
export const compareVersions = (a: string, b: string): number => {
	const pa = a
		.replace(/^v/, '')
		.split('.')
		.map((x) => parseInt(x, 10) || 0);
	const pb = b
		.replace(/^v/, '')
		.split('.')
		.map((x) => parseInt(x, 10) || 0);
	const n = Math.max(pa.length, pb.length);
	for (let i = 0; i < n; i++) {
		const da = pa[i] ?? 0;
		const db = pb[i] ?? 0;
		if (da !== db) return da - db;
	}
	return 0;
};

export type CheckStatus = 'idle' | 'checking' | 'up-to-date' | 'outdated' | 'error';

export interface CheckState {
	status: CheckStatus;
	latest?: string;
	error?: string;
}

/** 把任意 throw 值转成可读文本：Tauri invoke reject 抛出的是 { message, code, data } 对象而非 Error */
function describeError(e: unknown): string {
	if (e instanceof Error) return e.message;
	if (typeof e === 'string') return e;
	if (e && typeof e === 'object') {
		const msg = (e as { message?: unknown }).message;
		if (typeof msg === 'string' && msg) return msg;
		return JSON.stringify(e);
	}
	return String(e);
}

/**
 * 渠道更新检查器基类：统一状态机与错误处理。
 * 子类只负责实现 checkUpdate()——返回「最新/有更新」的结果状态，失败时抛错。
 */
abstract class Updater {
	readonly channelId: string;
	readonly channelUrl: string;
	state = $state<CheckState>({ status: 'idle' });

	constructor(channelId: string, channelUrl: string) {
		this.channelId = channelId;
		this.channelUrl = channelUrl;
	}

	async check() {
		this.state = { status: 'checking' };
		try {
			this.state = await this.checkUpdate();
		} catch (e) {
			this.state = { status: 'error', error: describeError(e) };
		}
	}

	/** 子类实现：检查是否有更新，返回最终结果状态；失败时抛错交由基类转 error */
	protected abstract checkUpdate(): Promise<CheckState>;

	reset() {
		this.state = { status: 'idle' };
	}
}

/** Play 渠道：Google Play 应用内更新插件。商店版本号（versionCode）无法转语义化版本，直接按「有/无更新」判定 */
class PlayUpdater extends Updater {
	constructor(channelUrl: string) {
		super('play', channelUrl);
	}

	protected async checkUpdate(): Promise<CheckState> {
		const { invoke } = await import('@tauri-apps/api/core');
		const info = (await invoke<PlayUpdateInfo>('plugin:app-update|check')) ?? {};
		return info.available || info.inProgress
			? { status: 'outdated', latest: String(info.versionCode ?? '') }
			: { status: 'up-to-date' };
	}
}

/** sideload 渠道（GitHub Release）：读取 release 资产 version.json，与本地版本比较 */
class GitHubUpdater extends Updater {
	constructor(channelUrl: string) {
		super('sideload', channelUrl);
	}

	protected async checkUpdate(): Promise<CheckState> {
		let latest = '';
		const errors: string[] = [];
		// Tauri 内走 Rust 侧 HTTP（无 CORS、自动跟随重定向）；浏览器里只能用原生 fetch
		const { fetch } = await import('@tauri-apps/plugin-http');
		for (const url of RELEASE_URLS) {
			try {
				const res = await fetch(url, { cache: 'no-store' });
				if (!res.ok) throw new Error(`HTTP ${res.status}`);
				const data = (await res.json()) as { version?: string };
				latest = String(data.version ?? '').replace(/^v/, '');
				if (latest) break;
				throw new Error('empty version field');
			} catch (e) {
				errors.push(`${url.split('/')[2]}: ${e instanceof Error ? e.message : String(e)}`);
			}
		}
		if (!latest) throw new Error(`all endpoints failed: ${errors.join('; ')}`);
		return compareVersions(latest, __APP_VERSION__) > 0
			? { status: 'outdated', latest }
			: { status: 'up-to-date' };
	}
}

/** 工厂：按渠道构造对应检查器；未知渠道回退 sideload（GitHub） */
function createUpdater(channel: string): Updater {
	const url = CHANNELS[channel] ?? CHANNELS.sideload;
	switch (channel) {
		case 'play':
			return new PlayUpdater(url);
		default:
			return new GitHubUpdater(url);
	}
}

/** 当前渠道的更新检查器：仅 Tauri 环境有意义，非 Tauri 时导出 null（调用方按需判空） */
export const updater: Updater | null = isTauri ? createUpdater(__CHANNEL__ || 'sideload') : null;
