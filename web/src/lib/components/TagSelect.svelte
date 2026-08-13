<script lang="ts">
	import Tag from './Tag.svelte';

	interface Props {
		/** key → 显示文本；插入顺序即展示顺序 */
		options: Record<string, string>;
		/** 已选 key 列表；空数组 = 不限制 */
		value: string[];
		class?: string;
	}

	let { class: clz, options, value = $bindable() }: Props = $props();

	function toggle(key: string) {
		value = value.includes(key) ? value.filter((k) => k !== key) : [...value, key];
	}
</script>

<div class="flex min-w-0 flex-wrap items-center gap-1 {clz}">
	{#each Object.entries(options) as [key, desc]}
		{@const active = value.includes(key)}
		<button aria-pressed={active} onclick={() => toggle(key)}>
			<Tag intereactive {active}>
				{desc}
			</Tag>
		</button>
	{/each}
</div>
