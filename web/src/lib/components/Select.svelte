<script lang="ts">
	import { Button, Dropdown, DropdownItem, Tooltip } from 'flowbite-svelte';
	import { ChevronDown } from '@lucide/svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		options: (string | Snippet)[];
		value: number;
		class?: string;
		tooltip?: string;
		disabled?: boolean;
		disabledValue?: number;
		disabledTooltip?: string;
	}

	let {
		disabled,
		disabledValue,
		disabledTooltip,
		tooltip,
		class: clz,
		options,
		value = $bindable()
	}: Props = $props();

	let isOpen = $state(false);
	let buttonWidth = $state(0);

	$effect(() => {
		if (disabled && disabledValue !== undefined) {
			value = disabledValue;
		}
	});
</script>

{#snippet renderDesc(desc: string | Snippet)}
	{#if typeof desc === 'string'}
		{desc}
	{:else}
		{@render desc()}
	{/if}
{/snippet}

<div bind:clientWidth={buttonWidth}>
	<Button
		{disabled}
		size="xs"
		color="alternative"
		class="gap-1 justify-start! cursor-pointer {clz}"
	>
		{@render renderDesc(options[value])}
		<ChevronDown class="h-3 w-3 ms-auto" />
	</Button>
	{#if disabled}
		{#if disabledTooltip}
			<Tooltip placement="top" class="text-xs p-1">{disabledTooltip}</Tooltip>
		{/if}
	{:else}
		{#if tooltip}
			<Tooltip placement="top" class="text-xs p-1">{tooltip}</Tooltip>
		{/if}
	{/if}
</div>
{#if !disabled}
	<Dropdown
		placement="bottom-start"
		class="list-none overflow-hidden!"
		style="width: {buttonWidth}px !important;"
		bind:isOpen
	>
		{#each options as desc, i}
			<DropdownItem
				class="cursor-pointer text-xs text-gray-700 dark:text-gray-200 
				{value == i ? 'bg-gray-100 dark:bg-gray-600 font-bold' : 'font-light'}"
				onclick={() => {
					value = i;
					isOpen = false;
				}}
			>
				{@render renderDesc(desc)}
			</DropdownItem>
		{/each}
	</Dropdown>
{/if}
