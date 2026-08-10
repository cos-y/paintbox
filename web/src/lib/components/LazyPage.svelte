<script lang="ts">
	import type { Component } from 'svelte';

	let { load }: { load: () => Promise<{ default: Component }> } = $props();
	let Page: Component | undefined = $state();

	// 组件在 layout load（含 wasm 初始化）完成后才渲染，此处的动态 import
	// 保证页面实现及其全部依赖的模块求值发生在 wasm 就绪之后。
	$effect(() => {
		void load().then((m) => (Page = m.default));
	});
</script>

{#if Page}
	<Page />
{/if}
