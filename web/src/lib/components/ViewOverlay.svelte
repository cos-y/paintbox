<script lang="ts">
	import { slide } from 'svelte/transition';
	import { drawer } from '$lib/drawer.svelte';
	import { setDrawerBackNeeded } from '$lib/back.svelte';

	const view = $derived(drawer.view);

	// 覆盖层打开时登记 back 需求（Android 根级注销后由系统接管退出）；关闭即清理
	$effect(() => {
		if (!view) return;
		setDrawerBackNeeded(true);
		return () => setDrawerBackNeeded(false);
	});
</script>

{#if view}
	<div
		class="absolute inset-0 z-50 bg-white dark:bg-gray-900"
		transition:slide={{ axis: 'x', duration: 250 }}
	>
		<svelte:component this={view.component} {...view.props} />
	</div>
{/if}
