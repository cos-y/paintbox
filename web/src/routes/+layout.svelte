<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { onMount } from 'svelte';
	import init, {init_searcher, search} from '../wasm-pkg/paintbox_wasm';

	let { children } = $props();
	let wasmReady = $state(false);

	const fetchData = fetch('/colors.csv').then(data => data.arrayBuffer());

	onMount(async () => {
		await init();
		init_searcher(new Uint8Array(await fetchData));
		wasmReady = true;
		let li = search(0x1189BD, 2, 10);
		console.log(li);
	})
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
{#if !wasmReady}
	<p>loading wasm..</p>
{:else}
	{@render children()}
{/if}
