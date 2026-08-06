// 更新检测：调用 GitHub Releases API 获取最新 tag 与当前版本比较。
// 渠道链接按 channel 映射（内置），无 channel（sideload）时视为 GitHub 渠道。
// 注意：GitHub API 未认证限 60 次/小时/IP，手动检查更新完全够用。

/** 最新版本来源：GitHub Release 的静态文件（releases/latest/download/version.json，CI 发版时自动上传）。
 * 不走 api.github.com（未认证限 60 次/小时），静态文件无 API 限流。 */
const RELEASE_URL =
	'https://github.com/cos-y/paintbox/releases/latest/download/version.json';

/**
 * 各渠道的更新/Changelog 页面（channel 由打包时 CHANNEL 环境变量注入；无则视为 sideload = GitHub 渠道）。
 * 只引导用户「查看更新内容」，不提供直接下载入口。
 */
const CHANNELS: Record<string, string> = {
	play: 'https://play.google.com/store/apps/details?id=com.cosy.paintbox',
	sideload: 'https://github.com/cos-y/paintbox/releases/latest'
	// 未来上架国内商店时加：huawei: 'https://appgallery.huawei.com/app/...', ...
};

/** 当前渠道 ID（无 channel 注入时为 sideload） */
export const channelId = __CHANNEL__ || 'sideload';

/** 当前渠道的更新/Changelog 页面（无 channel 注入时为 sideload / GitHub 渠道） */
export const channelUrl = CHANNELS[__CHANNEL__] ?? CHANNELS.sideload;

/** 语义化版本比较：a > b 返回 >0，a == b 返回 0 */
export const compareVersions = (a: string, b: string): number => {
	const pa = a.replace(/^v/, '').split('.').map((x) => parseInt(x, 10) || 0);
	const pb = b.replace(/^v/, '').split('.').map((x) => parseInt(x, 10) || 0);
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
}

class UpdateChecker {
	state = $state<CheckState>({ status: 'idle' });

	async check() {
		this.state = { status: 'checking' };
		try {
			const res = await fetch(RELEASE_URL, { cache: 'no-store' });
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const data = (await res.json()) as { version?: string };
			const latest = String(data.version ?? '').replace(/^v/, '');
			if (!latest) throw new Error('empty version');
			if (compareVersions(latest, __APP_VERSION__) > 0) {
				this.state = { status: 'outdated', latest };
			} else {
				this.state = { status: 'up-to-date' };
			}
		} catch {
			this.state = { status: 'error' };
		}
	}

	reset() {
		this.state = { status: 'idle' };
	}
}

export const updateChecker = new UpdateChecker();
