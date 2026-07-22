<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { Boxes, Search, Palette, Info } from '@lucide/svelte';
	import { Tooltip } from 'flowbite-svelte';

	let { children } = $props();

	const navs = [
		{
			title: 'stock',
			route: '/stock',
			svg: Boxes
		},
		{
			title: 'search',
			route: '/search',
			svg: Search
		},
		{
			title: 'palette',
			route: '/palette',
			svg: Palette
		}
	];
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
<!-- {#if navigating.to}
	navigating to {navigating.to.url.pathname}
{/if} -->
<!-- TODO: -->

<div class="flex h-dvh w-screen overflow-hidden">
	<aside class="w-16 bg-gray-50 dark:bg-gray-800 shrink-0 h-full overflow-y-auto">
		<ul class="w-full h-full flex flex-col py-2 space-y-1 overflow-hidden">
			{#each navs as { title, route, svg: Icon }}
				<li>
					<a
						class="flex w-full justify-center rounded-lg p-3 text-gray-500 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white"
						href={route}
					>
						<Icon />
					</a>
					<Tooltip placement="right">{title}</Tooltip>
				</li>
			{/each}
			<li class="mt-auto">
				<a
					class="flex w-full justify-center rounded-lg p-3 text-gray-500 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white"
					href="/about"
				>
					<!-- < /> -->
					<Info />
				</a>
				<Tooltip placement="right">about</Tooltip>
			</li>
		</ul>
	</aside>

	<main class="flex-1 h-full overflow-y-auto">
		{@render children()}
	</main>
</div>
