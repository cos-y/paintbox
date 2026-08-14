import { settings } from './settings.svelte';

/**
 * 主题应用：把 settings.theme（system/dark/light）解析为生效主题，
 * 同步 <html>.dark class、color-scheme 与 <meta name="theme-color">。
 * system 模式下监听系统主题变化实时跟随（移动端系统深色模式切换即时生效）。
 */

export type ResolvedTheme = 'dark' | 'light';

const THEME_COLORS: Record<ResolvedTheme, string> = {
	dark: '#111827', // gray-900
	light: '#ffffff'
};

const prefersDark = () =>
	typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches;

export const resolveTheme = (t: typeof settings.theme): ResolvedTheme =>
	t === 'system' ? (prefersDark() ? 'dark' : 'light') : t;

function apply() {
	const t = resolveTheme(settings.theme);
	const root = document.documentElement;
	root.classList.toggle('dark', t === 'dark');
	root.style.colorScheme = t;
	document
		.querySelector('meta[name="theme-color"]')
		?.setAttribute('content', THEME_COLORS[t]);
}

if (typeof window !== 'undefined') {
	const mq = window.matchMedia('(prefers-color-scheme: dark)');
	$effect.root(() => {
		// settings.theme 变化 → 重新应用
		$effect(() => {
			settings.theme;
			apply();
		});
		// system 模式下系统主题变化 → 实时跟随
		mq.addEventListener('change', apply);
	});
}
