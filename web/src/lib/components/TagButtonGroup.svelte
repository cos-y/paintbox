<script lang="ts">
	import { Tooltip } from 'flowbite-svelte';

	interface Props {
		/** key → 显示文本；插入顺序即展示顺序 */
		options: Record<string, string>;
		/** 当前选中 key（单选，必有值） */
		value: string;
		/** 选中变化（受控）：父组件回写 value */
		onchange: (value: string) => void;
		/** 禁用：整体半透明不可点；配 disabledTooltip 说明原因 */
		disabled?: boolean;
		/** 禁用时的悬停提示 */
		disabledTooltip?: string;
		/** 悬停提示 */
		tooltip?: string;
		class?: string;
	}

	let {
		tooltip,
		disabledTooltip,
		disabled = false,
		onchange,
		class: clz,
		options,
		value
	}: Props = $props();

	function select(key: string) {
		if (disabled || value === key) return;
		onchange(key);
	}
</script>

<!-- 分段按钮组：按钮连体单行，圆角只在外侧两端，相邻按钮用分割线；
     容器 w-max 不收缩，窄容器下由外层 overflow-x-auto 横向滚动 -->
<div class="flex w-max overflow-hidden rounded-md {disabled ? 'opacity-50' : ''}">
	{#each Object.entries(options) as [key, desc], i}
		{@const active = value === key}
		<button
			aria-pressed={active}
			{disabled}
			onclick={() => select(key)}
			class="flex cursor-pointer items-center px-2 py-0.5 text-xs transition-colors {clz} {active
				? 'bg-primary-500 text-white dark:bg-primary-500'
				: 'bg-gray-100 text-gray-600 enabled:hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-300 dark:enabled:hover:bg-gray-700'} {i >
			0
				? 'border-theme border-l'
				: ''}"
		>
			{desc}
		</button>
	{/each}
</div>
{#if disabled && disabledTooltip}
	<Tooltip placement="top" class="p-1 text-xs">{disabledTooltip}</Tooltip>
{:else if tooltip}
	<Tooltip placement="top" class="p-1 text-xs">{tooltip}</Tooltip>
{/if}
