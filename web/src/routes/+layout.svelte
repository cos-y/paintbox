<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { Package, Search, Palette, Info, Eclipse } from '@lucide/svelte';
	import { Tooltip } from 'flowbite-svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { t, type MessageKey } from '$lib/i18n.svelte';

	let { children } = $props();

	const navs: { key: MessageKey; route: string; svg: typeof Package }[] = [
		{ key: 'nav.stock', route: '/stock', svg: Package },
		{ key: 'nav.search', route: '/search', svg: Search },
		{ key: 'nav.gamut', route: '/gamut', svg: Eclipse },
		{ key: 'nav.about', route: '/about', svg: Info }
	];

	const isActive = (route: string) => page.url.pathname.startsWith(route);
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<meta name="theme-color" content="#3b82f6" />
</svelte:head>
<!-- {#if navigating.to}
	navigating to {navigating.to.url.pathname}
{/if} -->
<!-- TODO: -->

<div class="flex h-dvh w-screen overflow-hidden">
	<!-- desktop sidebar -->
	<aside class="hidden sm:block w-16 bg-gray-50 dark:bg-gray-800 shrink-0 h-full overflow-y-auto">
		<ul class="w-full h-full flex flex-col py-2 space-y-1 overflow-hidden">
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

	<main class="flex-1 h-full overflow-y-auto pt-[env(safe-area-inset-top)] pb-[calc(env(safe-area-inset-bottom)+3.5rem)] sm:pb-0">
		{@render children()}
	</main>

	<!-- mobile bottom nav -->
	<nav
		class="sm:hidden fixed bottom-0 left-0 right-0 z-50 border-t border-gray-200 bg-white pb-[env(safe-area-inset-bottom)] dark:border-gray-700 dark:bg-gray-900"
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
