<script lang="ts">
	import { MousePointerClick } from '@lucide/svelte';

	interface Props {
		hint: string;
	}

	let { hint }: Props = $props();
</script>

<!-- 右侧详情栏空态：引导用户点击左侧列表项查看详情 -->
<div class="flex h-full min-h-40 flex-col items-center justify-center gap-3 p-6 text-center">
	<MousePointerClick class="size-10 opacity-25" />
	<span class="text-xs text-gray-400 dark:text-gray-500">{hint}</span>
</div>
