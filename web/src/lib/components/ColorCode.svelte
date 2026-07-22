<script lang="ts">
	import { Copy } from '@lucide/svelte';
	import { Input, Tooltip } from 'flowbite-svelte';

	interface Props {
		re: string;
		text: string;
		oninput: (...vs: number[]) => void;
	}

	const { re, text, oninput }: Props = $props();
	const regexp = $derived(new RegExp(re));

	let localText = $state(text);
	let localParams: number[] = $state([]);

	const params = $derived.by(() => {
		const match = text.match(regexp);
		return match ? match.slice(1).map((x) => +x) : [];
	});

	const handleInput = (e: Event) => {
		const el = e.currentTarget! as HTMLInputElement;
		const match = el.value.match(regexp);
		if (match) {
			localParams = match.slice(1).map((x) => +x);
			oninput(...localParams);
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

<div class="relative">
	<Input
		class="text-xs! font-mono p-2 text-center w-full"
		type="text"
		name="rgb"
		pattern={re}
		autocomplete="off"
		autocorrect="off"
		autocapitalize="off"
		spellcheck="false"
		bind:value={localText}
		oninput={handleInput}
	/>
	<div class="absolute -right-px top-0 bottom-0 flex font-mono">
		<button
			class="flex-1 px-[10px] cursor-pointer text-gray-400 hover:text-gray-200
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
