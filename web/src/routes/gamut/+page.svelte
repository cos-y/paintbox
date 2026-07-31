<script lang="ts">
	import { listPaints, paintId, type PaintInfo } from '$lib/paints';
	import { stock } from '$lib/stock.svelte';
	import { Plus, X, Search, Package } from '@lucide/svelte';
	import { Button, Dropdown, DropdownItem } from 'flowbite-svelte';
	import { onMount, tick } from 'svelte';
	import { callWasm } from '$lib/wasmClient';
	import { Canvas } from '@threlte/core';
	import Scene from './scene.svelte';

	// ── paint catalog (cached once) ──
	const allPaints = listPaints();

	// ── source types ──
	interface ColorSource {
		id: string;
		type: 'color';
		text: string;
		rgb: number;
		valid: boolean;
	}

	interface PaintSource {
		id: string;
		type: 'paint';
		paint: PaintInfo | null;
		searchText: string;
	}

	interface StockSource {
		id: string;
		type: 'stock';
	}

	type Source = ColorSource | PaintSource | StockSource;

	let sources: Source[] = $state([]);
	let nextId = 0;
	let lastAddedId: string | null = $state(null);

	let gamut: string | undefined;
	let task: Promise<any>;

	const ndiv = 16;

	class SceneProps {
		matrices = $state(new Float32Array()) as Float32Array<ArrayBufferLike>;
		colors = $state(new Float32Array()) as Float32Array<ArrayBufferLike>;
	}
	const scene = new SceneProps();

	// ── derived: all rgb values fed into gamut ──
	let localColors = $state(new Set<number>());
	const colors = $derived.by(() => {
		const li = new Set<number>();
		for (const src of sources) {
			if (src.type === 'color' && src.valid) {
				li.add(src.rgb);
			} else if (src.type === 'paint' && src.paint) {
				li.add(src.paint.rgb);
			} else if (src.type === 'stock') {
				for (const paint of allPaints) {
					if (stock.has(paintId(paint))) {
						li.add(paint.rgb);
					}
				}
			}
		}
		return li;
	});

	$effect(() => {
		const updateScene = async (gamut: string) => {
			const [matrices, colors] = await Promise.all([
				callWasm<Float32Array>('gamut_matrices', [gamut]),
				callWasm<Float32Array>('gamut_colors', [gamut])
			]);
			scene.matrices = matrices;
			scene.colors = colors;
		};
		const remColors = localColors.difference(colors);
		if (remColors.size > 0 || task === undefined) {
			const fn = async () => {
				const newGamut = await callWasm<string>('new_gamut', [ndiv, new Uint32Array(colors)], {
					cancelInFlight: true
				});
				if (gamut !== undefined) {
					await callWasm<void>('free', [gamut]);
					console.log('Gamut::free');
				}
				gamut = newGamut;
				console.log('Gamut::new');
				await updateScene(newGamut);
			};
			task = fn();
		} else {
			const newColors = colors.difference(localColors);
			if (newColors.size > 0) {
				const fn = async (task: Promise<any>) => {
					await task;
					const modified = await callWasm<boolean>('gamut_insert_many', [
						gamut,
						new Uint32Array(newColors)
					]);
					console.log('Gamut::insert', modified);
					if (modified && gamut !== undefined) {
						await updateScene(gamut);
					}
				};
				task = fn(task);
			}
		}
		localColors = colors;
	});

	const hasStock = $derived(sources.some((s) => s.type === 'stock'));
	const stockCount = $derived(allPaints.filter((p) => stock.has(paintId(p))).length);

	// ── add / remove ──
	function addColor() {
		const id = String(nextId++);
		sources.push({ id, type: 'color', text: '', rgb: 0, valid: false });
		lastAddedId = id;
		tick().then(() => {
			(document.querySelector(`[data-card-id="${id}"] input`) as HTMLInputElement | null)?.focus();
		});
	}

	function addPaint() {
		const id = String(nextId++);
		sources.push({ id, type: 'paint', paint: null, searchText: '' });
		lastAddedId = id;
		tick().then(() => {
			(document.querySelector(`[data-card-id="${id}"] input`) as HTMLInputElement | null)?.focus();
		});
	}

	function addStock() {
		if (!hasStock) {
			sources.push({ id: String(nextId++), type: 'stock' });
		}
	}

	function removeSource(id: string) {
		sources = sources.filter((s) => s.id !== id);
	}

	// ── color card helpers ──
	function parseHexInput(raw: string): { text: string; rgb: number; valid: boolean } {
		let t = raw.trim();
		if (t.startsWith('#')) t = t.slice(1);
		else if (t.startsWith('0x') || t.startsWith('0X')) t = t.slice(2);
		if (/^[0-9a-fA-F]{3}$/.test(t)) {
			t = t[0] + t[0] + t[1] + t[1] + t[2] + t[2];
		}
		if (/^[0-9a-fA-F]{6}$/.test(t)) {
			return { text: raw, rgb: parseInt(t, 16), valid: true };
		}
		return { text: raw, rgb: 0, valid: false };
	}

	function updateColorHex(src: ColorSource, text: string) {
		const parsed = parseHexInput(text);
		src.text = parsed.text;
		src.rgb = parsed.rgb;
		src.valid = parsed.valid;
	}

	// ── paint fuzzy search ──
	function searchPaints(query: string): PaintInfo[] {
		if (!query || query.length < 1) return [];
		const q = query.toLowerCase();
		return allPaints
			.filter(
				(p) =>
					p.code.toLowerCase().includes(q) ||
					p.brand.toLowerCase().includes(q) ||
					p.desc.toLowerCase().includes(q) ||
					`${p.brand} ${p.code}`.toLowerCase().includes(q)
			)
			.slice(0, 20);
	}

	function selectPaint(src: PaintSource, paint: PaintInfo) {
		src.paint = paint;
		src.searchText = '';
	}

	function clearPaint(src: PaintSource) {
		src.paint = null;
		src.searchText = '';
	}

	// ── per-paint-card search state ──
	let paintResults: Record<string, PaintInfo[]> = $state({});
	let highlightedIdx: Record<string, number> = $state({});

	function updatePaintSearch(src: PaintSource, text: string) {
		src.searchText = text;
		paintResults[src.id] = searchPaints(text);
		highlightedIdx[src.id] = -1;
	}

	// ── add dropdown ──
	let addOpen = $state(false);

	// ── tab: redirect last input → add button ──
	function handleInputTab(e: KeyboardEvent, src: Source) {
		if (e.key === 'Tab' && !e.shiftKey && src.id === sources.at(-1)?.id) {
			e.preventDefault();
			let el: any = document.querySelector('[data-add-btn] button');
			el.focus();
		}
	}

	onMount(() => {
		addStock();
	});
</script>

<div class="flex h-full">
	<!-- ═══════ LEFT: 3D Canvas (placeholder) ═══════ -->
	<div class="flex-1 relative min-w-0 bg-gray-950">
		<Canvas>
			{@const { matrices, colors } = scene}
			<Scene {ndiv} {matrices} {colors} />
		</Canvas>
	</div>

	<!-- ═══════ RIGHT: Card Panel ═══════ -->
	<div
		class="w-86 shrink-0 flex flex-col border-l border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-900"
	>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			data-add-btn
			class="flex items-center justify-between border-b border-gray-200 px-3 py-2 dark:border-gray-700"
		>
			<span class="text-sm font-semibold text-gray-700 dark:text-gray-200">Source Colors</span>

			<Button
				size="xs"
				color="alternative"
				class="cursor-pointer"
				onkeydown={(e: any) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault();
						addOpen = !addOpen;
					}
				}}
			>
				<Plus class="h-3.5 w-3.5" />
				Add
			</Button>
			<Dropdown placement="bottom-end" class="list-none overflow-hidden!" bind:isOpen={addOpen}>
				<DropdownItem
					class="cursor-pointer text-xs text-gray-700 dark:text-gray-200"
					onclick={() => {
						addColor();
						addOpen = false;
					}}
				>
					<span class="inline-flex items-center gap-2">
						<span
							class="h-3.5 w-3.5 rounded-full border border-black/15 bg-gradient-to-br from-red-400 via-green-400 to-blue-500"
						></span>
						Color
					</span>
				</DropdownItem>
				<DropdownItem
					class="cursor-pointer text-xs text-gray-700 dark:text-gray-200"
					onclick={() => {
						addPaint();
						addOpen = false;
					}}
				>
					<span class="inline-flex items-center gap-2">
						<Search class="h-3.5 w-3.5" />
						Paint
					</span>
				</DropdownItem>
				<DropdownItem
					class="cursor-pointer text-xs text-gray-700 dark:text-gray-200"
					onclick={() => {
						addStock();
						addOpen = false;
					}}
					disabled={hasStock}
				>
					<span class="inline-flex items-center gap-2">
						<Package class="h-3.5 w-3.5" />
						My Stock
					</span>
				</DropdownItem>
			</Dropdown>
		</div>

		{#snippet colorCard(src: any)}
			<div class="flex items-center gap-2">
				<div
					class="h-7 w-7 shrink-0 rounded-md border border-black/10"
					class:opacity-50={!src.valid}
					style="background-color: {src.valid
						? '#' + src.rgb.toString(16).padStart(6, '0')
						: '#e5e5e5'}"
				></div>
				<div class="min-w-0 flex-1">
					<input
						type="text"
						class="w-full rounded-md border border-gray-300 bg-white py-1 px-2 text-xs text-gray-900 outline-none focus:border-primary-500 focus:ring-1 focus:ring-primary-500 placeholder:text-gray-400 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 dark:placeholder:text-gray-500"
						placeholder="#ff0000"
						maxlength="9"
						spellcheck="false"
						value={src.text}
						oninput={(e) => updateColorHex(src, (e.target as HTMLInputElement).value)}
						onkeydown={(e) => handleInputTab(e, src)}
					/>
				</div>
			</div>
		{/snippet}

		{#snippet paintCard(src: any)}
			{#if src.paint}
				<div class="flex items-center gap-2">
					<div
						class="h-7 w-7 shrink-0 rounded-md border border-black/10"
						style="background-color: #{src.paint.rgb.toString(16).padStart(6, '0')}"
					></div>
					<div class="min-w-0 flex-1 text-xs">
						<div class="font-semibold uppercase">
							{src.paint.brand}/{src.paint.code}
						</div>
						<div class="truncate text-gray-500 dark:text-gray-400">
							{src.paint.desc}
						</div>
					</div>
					<button
						type="button"
						class="cursor-pointer text-[10px] text-gray-400 hover:text-primary-500"
						tabindex="-1"
						onclick={() => clearPaint(src)}
					>
						change
					</button>
				</div>
			{:else}
				<div class="min-w-0">
					<div class="relative">
						<Search
							class="pointer-events-none absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-gray-400"
						/>
						<input
							type="text"
							class="w-full rounded-md border border-gray-300 bg-white py-1 pl-7 pr-2 text-xs text-gray-900 outline-none focus:border-primary-500 focus:ring-1 focus:ring-primary-500 placeholder:text-gray-400 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 dark:placeholder:text-gray-500"
							placeholder="search brand / code / name..."
							value={src.searchText}
							oninput={(e) => updatePaintSearch(src, (e.target as HTMLInputElement).value)}
							onblur={() => {
								setTimeout(() => {
									paintResults[src.id] = [];
									highlightedIdx[src.id] = -1;
								}, 150);
							}}
							onkeydown={(e) => {
								if (e.key === 'Tab') {
									handleInputTab(e, src);
									return;
								}
								const items = paintResults[src.id];
								if (!items?.length) return;
								let idx = highlightedIdx[src.id] ?? -1;
								if (e.key === 'ArrowDown') {
									e.preventDefault();
									idx = idx + 1 >= items.length ? 0 : idx + 1;
									highlightedIdx[src.id] = idx;
									requestAnimationFrame(() => {
										(e.target as HTMLElement)
											.closest('[data-card-id]')
											?.querySelector('[data-hl]')
											?.scrollIntoView({ block: 'nearest' });
									});
								} else if (e.key === 'ArrowUp') {
									e.preventDefault();
									idx = idx <= 0 ? items.length - 1 : idx - 1;
									highlightedIdx[src.id] = idx;
									requestAnimationFrame(() => {
										(e.target as HTMLElement)
											.closest('[data-card-id]')
											?.querySelector('[data-hl]')
											?.scrollIntoView({ block: 'nearest' });
									});
								} else if (e.key === 'Enter') {
									e.preventDefault();
									if (idx >= 0 && idx < items.length) selectPaint(src, items[idx]);
								} else if (e.key === 'Escape') {
									paintResults[src.id] = [];
									highlightedIdx[src.id] = -1;
								}
							}}
						/>
					</div>
					{#if paintResults[src.id]?.length}
						<div
							class="absolute z-20 mt-1 max-h-48 w-[calc(100%-2rem)] overflow-y-auto rounded-md border border-gray-200 bg-white shadow-lg dark:border-gray-600 dark:bg-gray-700"
						>
							{#each paintResults[src.id] as p, i}
								{@const hl = (highlightedIdx[src.id] ?? -1) === i}
								<button
									type="button"
									data-hl={hl ? '' : undefined}
									class="flex w-full cursor-pointer items-center gap-2 px-2.5 py-1.5 text-left text-xs {hl
										? 'bg-gray-100 dark:bg-gray-600'
										: 'hover:bg-gray-100 dark:hover:bg-gray-600'}"
									onclick={() => selectPaint(src, p)}
								>
									<span
										class="h-4 w-4 shrink-0 rounded-sm border border-black/10"
										style="background-color: #{p.rgb.toString(16).padStart(6, '0')}"
									></span>
									<span class="font-semibold uppercase">{p.brand}/{p.code}</span>
									<span class="truncate text-gray-500 dark:text-gray-400">{p.desc}</span>
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{/if}
		{/snippet}

		{#snippet stockCard(src: any)}
			<div class="flex items-center gap-2">
				<div
					class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary-100 text-primary-600 dark:bg-primary-900 dark:text-primary-300"
				>
					<Package class="h-4 w-4" />
				</div>
				<div class="min-w-0 flex-1 text-xs">
					<div class="font-semibold text-gray-700 dark:text-gray-200">My Stock</div>
					<div class="text-gray-500 dark:text-gray-400">
						{stockCount} paint{stockCount !== 1 ? 's' : ''}
					</div>
				</div>
			</div>
		{/snippet}

		<!-- card list -->
		<div class="flex-1 space-y-2 overflow-y-auto p-2">
			{#each sources as src (src.id)}
				{@const valid =
					src.type === 'stock' ? true : src.type === 'color' ? src.valid : src.paint !== null}
				<div
					class="relative rounded-lg border p-2.5 pr-7 transition {valid
						? 'border-gray-200 bg-gray-50 dark:border-gray-700 dark:bg-gray-800'
						: 'border-dashed border-gray-300 bg-gray-100/70 dark:border-gray-600 dark:bg-gray-800/60'}"
					data-card-id={src.id}
				>
					<button
						type="button"
						class="absolute top-1.5 right-1.5 cursor-pointer rounded p-0.5 text-gray-400 hover:bg-gray-200 hover:text-gray-600 dark:hover:bg-gray-700 dark:hover:text-gray-300"
						tabindex="-1"
						onclick={() => removeSource(src.id)}
					>
						<X class="h-3.5 w-3.5" />
					</button>

					{#if src.type === 'color'}
						{@render colorCard(src)}
					{:else if src.type === 'paint'}
						{@render paintCard(src)}
					{:else if src.type === 'stock'}
						{@render stockCard(src)}
					{/if}
				</div>
			{:else}
				<div class="flex flex-col items-center justify-center py-12 text-center text-gray-400">
					<div class="mb-2 text-sm">No sources yet</div>
					<div class="text-xs">Click <span class="font-medium">Add</span> to start</div>
				</div>
			{/each}
		</div>

		<div
			class="border-t border-gray-200 px-3 py-1.5 text-[11px] text-gray-400 dark:border-gray-700"
		>
			{colors.size} color{colors.size !== 1 ? 's' : ''} in gamut
		</div>
	</div>
</div>
