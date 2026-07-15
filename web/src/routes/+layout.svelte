<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { Boxes, Search, Palette } from 'lucide-svelte';

	// import { navigating } from '$app/state';
	import { search } from '../wasm-pkg/paintbox_wasm';

	let { children } = $props();

	// let li = search(0x1189bd, 2, 10);
	// console.log(li);

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

<div class="flex h-screen w-screen overflow-hidden">
	<aside class="w-16 bg-base-300 shrink-0 h-full overflow-y-auto">
		<ul class="dx-menu w-full grow">
			<!-- bg-base-200 rounded-box -->
			{#each navs as { title, route, svg: Icon }}
				<li>
					<a class="dx-tooltip dx-tooltip-right block w-full" href={route} data-tip={title}>
						<Icon class="mx-auto" />
					</a>
				</li>
			{/each}
		</ul>
	</aside>

	<main class="flex-1 h-full overflow-y-auto">
		{@render children()}
	</main>
</div>
