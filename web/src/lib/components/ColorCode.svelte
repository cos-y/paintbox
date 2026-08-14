<script lang="ts">
	import { Copy } from '@lucide/svelte';
	import { Input, Tooltip } from 'flowbite-svelte';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		re: string;
		text: string;
		oninput?: (...vs: string[]) => void;
		onfocus?: (e: FocusEvent) => void;
		/** extra legality check on the parsed params (e.g. range bounds); false -> red ring, no oninput */
		validate?: (params: string[]) => boolean;
		class?: string;
		inputClass?: string;
		readonly?: boolean;
	}

	const {
		re,
		text,
		oninput,
		onfocus,
		validate,
		class: clz,
		readonly,
		inputClass
	}: Props = $props();
	const regexp = $derived(new RegExp(re));

	// Text shown in the box. While focused (editing) it keeps the user's input;
	// on blur it is re-synced to the canonical `text`.
	let localText = $state(text);
	let focused = $state(false);
	let error = $state(false);

	const handleInput = (e: Event) => {
		const el = e.currentTarget! as HTMLInputElement;
		const match = el.value.match(regexp);
		if (match && (validate === undefined || validate(match.slice(1)))) {
			error = false;
			oninput?.(...match.slice(1));
		} else {
			error = true;
		}
	};

	const handleFocus = (e: FocusEvent) => {
		focused = true;
		onfocus?.(e);
	};

	const handleBlur = () => {
		focused = false;
		error = false;
		localText = text;
	};

	$effect(() => {
		if (!focused && !error) {
			localText = text;
		}
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
		class={`p-2 text-center font-mono text-xs! ${inputClass} w-full${
			error ? ' border-red-500! ring-2! ring-red-500!' : ''
		}`}
		type="text"
		name="rgb"
		pattern={re}
		autocomplete="off"
		autocorrect="off"
		autocapitalize="off"
		spellcheck="false"
		bind:value={localText}
		oninput={handleInput}
		onfocus={handleFocus}
		onblur={handleBlur}
		{readonly}
	/>
	<div class="absolute top-1/2 right-0 flex -translate-y-1/2 font-mono">
		<button
			class="cursor-pointer px-2 py-2 text-gray-500 outline-offset-0
				hover:text-gray-700 focus:rounded-lg focus:outline-2 focus:outline-primary-500 dark:text-gray-400 dark:hover:text-gray-200"
			onclick={handleCopy}
			onmouseenter={handleMouseEnter}
		>
			<Copy size="1rem" />
		</button>
		<Tooltip placement="bottom" class="text-xs"
			>{isCopied ? t('colorCode.copied') : t('colorCode.copy')}</Tooltip
		>
	</div>
</div>
