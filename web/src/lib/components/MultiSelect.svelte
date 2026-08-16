<script lang="ts">
	import { Badge, Button, Checkbox, Dropdown, DropdownItem, Tooltip } from 'flowbite-svelte';
	import { ChevronDown } from '@lucide/svelte';

	interface Props {
		title: string;
		options: { [_: string]: string };
		value: string[];
		class?: string;
		tooltip?: string;
	}

	let { tooltip, class: clz, title, options, value = $bindable() }: Props = $props();

	let isOpen = $state(false);
</script>

<div>
	<Button size="xs" color="alternative" class="relative cursor-pointer justify-start! gap-1 {clz}">
		{value.length == 0 ? title : options[value[0]]}
		{#if value.length > 1}
			<Badge
				class="absolute top-1.5 right-7 rounded-full bg-primary-500 pr-1.5 pl-1.5 text-xs dark:bg-primary-500 dark:text-white"
				>{value.length}
			</Badge>
		{/if}
		<ChevronDown class="ms-auto h-3 w-3" />
	</Button>
	{#if tooltip}
		<Tooltip placement="top" class="p-1 text-xs">{tooltip}</Tooltip>
	{/if}
</div>
<Dropdown
	placement="bottom-start"
	class="cursor-pointer! list-none overflow-hidden! {clz}"
	bind:isOpen
>
	{#each Object.entries(options) as [key, desc] (key)}
		{@const idx = value.indexOf(key)}
		<DropdownItem
			class="cursor-pointer"
			onclick={() => {
				const li = [...value];
				if (idx != -1) {
					li.splice(idx, 1);
				} else {
					li.push(key);
				}
				value = li;
				isOpen = true;
			}}
		>
			<div
				class="pointer-events-none w-full [&_label]:text-xs!
				{idx != -1 ? '[&_label]:font-bold!' : '[&_label]:font-light!'}"
			>
				<Checkbox class="text-primary-500" checked={idx != -1}>
					{desc}
				</Checkbox>
			</div>
		</DropdownItem>
	{/each}
</Dropdown>
