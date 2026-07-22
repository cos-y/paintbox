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
	<Button size="xs" color="alternative" class="relative gap-1 justify-start! cursor-pointer {clz}">
		{value.length == 0 ? title : options[value[0]]}
		{#if value.length > 1}
			<Badge
				class="absolute pl-1.5 pr-1.5 text-xs top-1.5 right-7 rounded-full bg-primary-500 dark:bg-primary-500 dark:text-white"
				>{value.length}
			</Badge>
		{/if}
		<ChevronDown class="h-3 w-3 ms-auto" />
	</Button>
	{#if tooltip}
		<Tooltip placement="top" class="text-xs p-1">{tooltip}</Tooltip>
	{/if}
</div>
<Dropdown
	placement="bottom-start"
	class="list-none overflow-hidden! cursor-pointer! {clz}"
	bind:isOpen
>
	{#each Object.entries(options) as [key, desc]}
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
