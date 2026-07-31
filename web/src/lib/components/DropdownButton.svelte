<script lang="ts">
	import { Button, Dropdown, DropdownItem, type DropdownProps } from 'flowbite-svelte';
	import type { Snippet } from 'svelte';

	interface Option {
		disabled?: boolean;
		onclick?: () => void;
		children: Snippet;
	}

	interface Props extends DropdownProps {
		children: Snippet;
		options: Option[];
	}

	let { isOpen = $bindable(false), children, options, ...dropdownProps }: Props = $props();
</script>

<Button
	size="xs"
	color="alternative"
	class="cursor-pointer"
	onkeydown={(e: any) => {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			isOpen = !isOpen;
		}
	}}
>
	{@render children()}
</Button>
<Dropdown {...dropdownProps} class="list-none overflow-hidden!" bind:isOpen>
	{#each options as { onclick, children, disabled = false }}
		<DropdownItem
			class="cursor-pointer text-xs text-gray-700 dark:text-gray-200"
			onclick={() => {
				onclick?.();
				isOpen = false;
			}}
			{disabled}
		>
			<span class="inline-flex items-center gap-2">
				{@render children()}
			</span>
		</DropdownItem>
	{/each}
</Dropdown>
