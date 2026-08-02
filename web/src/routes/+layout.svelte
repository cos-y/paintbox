<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { Package, Search, Palette, Info, Eclipse } from '@lucide/svelte';
	import { Tooltip } from 'flowbite-svelte';
	import { page } from '$app/state';

	let { children } = $props();

	const navs = [
		{ title: 'stock', route: '/stock', svg: Package },
		{ title: 'search', route: '/search', svg: Search },
		{ title: 'gamut', route: '/gamut', svg: Eclipse },
		{ title: 'about', route: '/about', svg: Info }
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
			{#each navs as { title, route, svg: Icon }, i}
				{@const active = isActive(route)}
				<li class={i == navs.length - 1 ? 'mt-auto' : ''}>
					<a
						class="flex w-full justify-center rounded-lg p-3 transition-colors {active
							? 'bg-gray-200 text-primary-600 dark:bg-gray-700 dark:text-white'
							: 'text-gray-500 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white'}"
						href={route}
					>
						<Icon />
					</a>
					<Tooltip placement="right">{title}</Tooltip>
				</li>
			{/each}
		</ul>
	</aside>

	<main class="flex-1 h-full overflow-y-auto pb-14 sm:pb-0">
		{@render children()}
	</main>

	<!-- mobile bottom nav -->
	<nav
		class="sm:hidden fixed bottom-0 left-0 right-0 z-50 border-t border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-900"
	>
		<div class="mx-auto flex h-14 max-w-lg items-center justify-around">
			{#each navs as { title, route, svg: Icon }}
				{@const active = isActive(route)}
				<a
					href={route}
					class="flex flex-col items-center gap-0.5 px-3 py-1 transition-colors {active
						? 'text-primary-600 dark:text-primary-400'
						: 'text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white'}"
				>
					<Icon class="size-5" />
					<span class="text-[10px] leading-none font-medium">{title}</span>
				</a>
			{/each}
		</div>
	</nav>
</div>
