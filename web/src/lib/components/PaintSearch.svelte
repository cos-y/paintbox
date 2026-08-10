<script lang="ts">
	import { Search } from '@lucide/svelte';
	import { listPaints, type PaintInfo } from '$lib/paints.svelte';
	import { paintDesc } from '$lib/i18ndyn.svelte';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		onselect: (paint: PaintInfo) => void;
		/** 取消（Esc）时回调，便于外部收回弹层 */
		oncancel?: () => void;
	}

	let { onselect, oncancel }: Props = $props();

	const allPaints = listPaints();

	let text = $state('');
	let results: PaintInfo[] = $state([]);
	let highlighted = $state(-1);
	let inputEl: HTMLInputElement | undefined = $state();

	// 挂载即聚焦（弹层场景直接输入）
	$effect(() => {
		inputEl?.focus();
	});

	const searchPaints = (query: string): PaintInfo[] => {
		if (!query) return [];
		const q = query.toLowerCase();
		return allPaints
			.filter(
				(p) =>
					p.code.toLowerCase().includes(q) ||
					p.brand.toLowerCase().includes(q) ||
					paintDesc(p).toLowerCase().includes(q) ||
					`${p.brand} ${p.code}`.toLowerCase().includes(q)
			)
			.slice(0, 20);
	};

	const handleInput = (e: Event) => {
		text = (e.target as HTMLInputElement).value;
		results = searchPaints(text);
		// 有结果时默认高亮第一项，与 Enter 默认选中的心智一致
		highlighted = results.length > 0 ? 0 : -1;
	};

	const select = (p: PaintInfo) => {
		onselect(p);
		text = '';
		results = [];
		highlighted = -1;
	};

	const scrollHighlighted = (e: KeyboardEvent) => {
		requestAnimationFrame(() => {
			(e.target as HTMLElement)
				.closest('.paint-search')
				?.querySelector('[data-hl]')
				?.scrollIntoView({ block: 'nearest' });
		});
	};

	const handleKeydown = (e: KeyboardEvent) => {
		if (!results.length) return;
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			highlighted = highlighted + 1 >= results.length ? 0 : highlighted + 1;
			scrollHighlighted(e);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			highlighted = highlighted <= 0 ? results.length - 1 : highlighted - 1;
			scrollHighlighted(e);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			// 未用方向键选择时默认取第一项
			const idx = highlighted >= 0 ? highlighted : 0;
			if (idx < results.length) select(results[idx]);
		} else if (e.key === 'Escape') {
			e.preventDefault();
			results = [];
			highlighted = -1;
			oncancel?.();
		}
	};
</script>

<div class="paint-search relative">
	<Search
		class="pointer-events-none absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-gray-500"
	/>
	<input
		type="text"
		class="h-9 w-full rounded-lg border border-gray-200 bg-gray-100 pr-2 pl-7 text-sm text-gray-900 outline-none placeholder:text-gray-500 focus:border-primary-500 focus:outline-none dark:border-gray-700 dark:bg-gray-800 dark:text-white dark:placeholder:text-gray-400"
		placeholder={t('gamut.searchPlaceholder')}
		value={text}
		bind:this={inputEl}
		oninput={handleInput}
		onkeydown={handleKeydown}
		onblur={() => {
			setTimeout(() => {
				results = [];
				highlighted = -1;
			}, 150);
		}}
	/>
	{#if results.length}
		<div
			class="absolute z-20 mt-1 max-h-48 w-full min-w-64 overflow-y-auto rounded-lg border border-gray-200 bg-white shadow-lg dark:border-gray-600 dark:bg-gray-700"
		>
			{#each results as p, i}
				{@const hl = highlighted === i}
				<button
					type="button"
					data-hl={hl ? '' : undefined}
					class="flex w-full cursor-pointer items-center gap-2 px-2.5 py-1.5 text-left text-xs {hl
						? 'bg-gray-100 dark:bg-gray-600'
						: 'hover:bg-gray-100 dark:hover:bg-gray-600'}"
					onclick={() => select(p)}
				>
					<span
						class="h-4 w-4 shrink-0 rounded-sm border border-black/10"
						style="background-color: #{p.rgb.toString(16).padStart(6, '0')}"
					></span>
					<span class="font-semibold uppercase">{p.brand}/{p.code}</span>
					<span class="truncate text-gray-500 dark:text-gray-400">{paintDesc(p)}</span>
				</button>
			{/each}
		</div>
	{/if}
</div>
