<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { Package, Search, Info, Eclipse } from '@lucide/svelte';
	import { Tooltip } from 'flowbite-svelte';
	import { page } from '$app/state';
	import { goto, beforeNavigate } from '$app/navigation';
	import { t, type MessageKey } from '$lib/i18n.svelte';
	import { isTauri } from '@tauri-apps/api/core';
	import { drawer } from '$lib/drawer.svelte';
	import Drawer from '$lib/components/Drawer.svelte';
	import { isMedia } from '$lib/utils.svelte';

	let { children } = $props();

	// 字体：JetBrains Mono latin 子集由 layout.css 的 @font-face 声明，全平台生效

	// 任何导航（切段/跳转/浏览器返回）都关闭视图覆盖层：播关闭动画（与页面切换并行）
	beforeNavigate(() => drawer.closeAnimated());

	const navs: { key: MessageKey; route: string; svg: typeof Package }[] = [
		{ key: 'nav.stock', route: '/stock', svg: Package },
		{ key: 'nav.search', route: '/search', svg: Search },
		{ key: 'nav.gamut', route: '/gamut', svg: Eclipse },
		{ key: 'nav.about', route: '/about', svg: Info }
	];

	const isActive = (route: string) => page.url.pathname.startsWith(route);

	// Tauri 环境禁止双指缩放（WebView 默认允许 pinch-zoom，原生手势优先）
	$effect(() => {
		if (!isTauri()) return;
		// 原生 app 体验：默认禁用光标文本选择（输入框除外，CSS 在 layout.css）
		document.documentElement.classList.add('tauri-no-select');
		// 双指触摸时阻止默认行为（pinch 缩放）
		const preventPinch = (e: TouchEvent) => {
			if (e.touches.length > 1) e.preventDefault();
		};
		// iOS WebView 的 gesturestart（传统手势事件，双保险）
		const preventGesture = ((e: Event) => e.preventDefault()) as EventListener;
		document.addEventListener('touchmove', preventPinch, { passive: false });
		document.addEventListener('gesturestart', preventGesture, { passive: false });
		// 同步收紧 viewport（Android WebView 会遵守 user-scalable=no）
		const vp = document.querySelector('meta[name="viewport"]');
		if (vp) {
			vp.setAttribute(
				'content',
				'width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, viewport-fit=cover'
			);
		}
		return () => {
			document.removeEventListener('touchmove', preventPinch);
			document.removeEventListener('gesturestart', preventGesture);
		};
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<meta name="theme-color" content="#3b82f6" />
</svelte:head>
<!-- {#if navigating.to}
	navigating to {navigating.to.url.pathname}
{/if} -->
<!-- TODO: -->

<div class="relative flex h-dvh w-screen overflow-hidden">
	<!-- desktop sidebar -->
	<aside class="hidden h-full w-16 shrink-0 overflow-y-auto bg-gray-50 sm:block dark:bg-gray-800">
		<ul class="flex h-full w-full flex-col space-y-1 overflow-hidden py-2">
			{#each navs as { key, route, svg: Icon }, i}
				{@const active = isActive(route)}
				<li class={i == navs.length - 1 ? 'mt-auto' : ''}>
					<a
						class="flex w-full justify-center rounded-lg p-3 transition-colors {active
							? 'bg-gray-200 text-primary-600 dark:bg-gray-700 dark:text-white'
							: 'text-gray-500 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white'}"
						href={route}
						onclick={(e) => {
							e.preventDefault();
							// 区段切换不压历史：回退不会跨区段跳转，区段顶层回退即退出应用
							goto(route, { replaceState: true, noScroll: true });
						}}
					>
						<Icon />
					</a>
					<Tooltip placement="right">{t(key)}</Tooltip>
				</li>
			{/each}
		</ul>
	</aside>

	<main
		class="h-full flex-1 overflow-x-hidden overflow-y-auto pt-[env(safe-area-inset-top)] pb-[calc(env(safe-area-inset-bottom)+3.5rem)] sm:pb-0"
	>
		{@render children()}
	</main>

	<!-- 全局视图栈：手机端底部卡片（桌面端用页面内常驻详情栏，不再有覆盖层） -->
	{#if !isMedia().sm}
		<Drawer />
	{/if}

	<!-- mobile bottom nav -->
	<nav
		class="fixed right-0 bottom-0 left-0 z-50 border-t border-gray-200 bg-white pb-[env(safe-area-inset-bottom)] sm:hidden dark:border-gray-700 dark:bg-gray-900"
	>
		<div class="mx-auto flex h-14 max-w-lg items-center justify-around">
			{#each navs as { key, route, svg: Icon }}
				{@const active = isActive(route)}
				<a
					href={route}
					class="flex flex-col items-center gap-0.5 px-3 py-1 transition-colors {active
						? 'text-primary-600 dark:text-primary-400'
						: 'text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white'}"
					onclick={(e) => {
						e.preventDefault();
						goto(route, { replaceState: true, noScroll: true });
					}}
				>
					<Icon class="size-5" />
					<span class="text-[10px] leading-none font-medium">{t(key)}</span>
				</a>
			{/each}
		</div>
	</nav>
</div>
