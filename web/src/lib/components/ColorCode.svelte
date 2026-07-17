<script lang="ts">
	import { Copy } from 'lucide-svelte';
	import { tick } from 'svelte';
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

<div class="color-code">
	<Input
		class="text-xs! font-mono p-2 text-center"
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
	<div class="copy-code flex font-mono">
		<button
			aria-label="copy code"
			type="button"
			class="flex-1"
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
