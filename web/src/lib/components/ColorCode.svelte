<script lang="ts">
	import { Copy } from 'lucide-svelte';
	import { tick } from 'svelte';

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

	const handleInput = (e: Event & { currentTarget: HTMLInputElement }) => {
		const match = e.currentTarget.value.match(regexp);
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
	<input
		class="text-sm! font-mono"
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
	<div
		role="button"
		tabindex="-1"
		class="
			copy-code flex font-mono
			dx-tooltip dx-tooltip-bottom before:text-xs {isCopied ? 'before:text-green-300' : ''}"
		data-tip={isCopied ? 'copied!' : 'copy to clipboard'}
		onmouseenter={handleMouseEnter}
	>
		<button aria-label="copy code" type="button" class="flex-1" onclick={handleCopy}>
			<Copy size="1rem" color="#666" />
		</button>
	</div>
</div>
