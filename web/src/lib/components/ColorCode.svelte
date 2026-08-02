<script lang="ts">
	import { Copy } from '@lucide/svelte';
	import { Input, Tooltip } from 'flowbite-svelte';

	interface Props {
		re: string;
		text: string;
		oninput?: (...vs: string[]) => void;
		onfocus?: (e: FocusEvent) => void;
		class?: string;
		readonly?: boolean;
		textAlign?: 'left' | 'center' | 'right';
	}

	const {
		re,
		text,
		oninput,
		onfocus,
		class: clz,
		readonly,
		textAlign = 'center'
	}: Props = $props();
	const regexp = $derived(new RegExp(re));

	let localText = $state(text);
	let localParams: string[] = $state([]);

	const params = $derived.by(() => {
		const match = text.match(regexp);
		return match ? match.slice(1) : [];
	});

	const handleInput = (e: Event) => {
		if (oninput !== undefined) {
			const el = e.currentTarget! as HTMLInputElement;
			const match = el.value.match(regexp);
			if (match) {
				localParams = match.slice(1);
				oninput(...localParams);
			}
		}
	};

	$effect(() => {
		if (params.length == localParams.length && params.every((x, i) => x == localParams[i])) {
			return;
		}
		localText = text;
	});

	let isCopied = $state(false);

	const handleCopy = () => {
		navigator.clipboard.writeText(localText);
		isCopied = true;
	};

	const handleMouseEnter = () => {
		isCopied = false;
	};
</script>

<div class="relative {clz}">
	<Input
		class={`text-xs! font-mono p-2 text-${textAlign} w-full`}
		type="text"
		name="rgb"
		pattern={re}
		autocomplete="off"
		autocorrect="off"
		autocapitalize="off"
		spellcheck="false"
		bind:value={localText}
		oninput={handleInput}
		{onfocus}
		{readonly}
	/>
	<div class="absolute right-0 top-1/2 -translate-y-1/2 flex font-mono">
		<button
			class="px-2 py-2 cursor-pointer text-gray-400 hover:text-gray-200
				outline-offset-0 focus:rounded-lg focus:outline-2 focus:outline-primary-500"
			onclick={handleCopy}
			onmouseenter={handleMouseEnter}
		>
			<Copy size="1rem" />
		</button>
		<Tooltip placement="bottom" class="text-xs"
			>{isCopied ? 'copied!' : 'copy to clipboard'}</Tooltip
		>
	</div>
</div>
