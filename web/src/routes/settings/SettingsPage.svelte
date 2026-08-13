<script lang="ts">
	import Github from '$lib/icons/Github.svelte';
	import Qq from '$lib/icons/Qq.svelte';
	import Discord from '$lib/icons/Discord.svelte';
	import PlayStore from '$lib/icons/PlayStore.svelte';
	import {
		ShieldCheck,
		EyeOff,
		Info,
		TriangleAlert,
		Coffee,
		Handshake,
		Moon,
		ChevronDown,
		Settings as SettingsIcon
	} from '@lucide/svelte';
	import { Button, Toggle } from 'flowbite-svelte';
	import { slide } from 'svelte/transition';
	import favicon from '$lib/assets/favicon.svg';
	import { i18n, t, type Locale } from '$lib/i18n.svelte';
	import { settings } from '$lib/settings.svelte';
	import { updater } from '$lib/update.svelte';
	import Select from '$lib/components/Select.svelte';
	import { isTauri } from '@tauri-apps/api/core';
	import { isMedia, openExternal } from '$lib/utils.svelte';

	const githubUrl = 'https://github.com/cos-y/paintbox';
	const discordUrl = 'https://discord.gg/QG2ZdVRkxN'; // TODO: 替换为实际邀请链接
	const qqUrl = isTauri()
		? 'mqqapi://card/show_pslcard?src_type=internal&version=1&uin=963504621'
		: 'https://qm.qq.com/q/3bWtHScQUo';
	const donateUrl = 'https://afdian.com/a/cos_y';

	// 语言选项。ja/es 需要先在 i18n.svelte.ts 中补齐对应字典，再置 available: true
	const LOCALES = [
		{ code: 'en', label: 'English', available: true },
		{ code: 'zh', label: '中文', available: true },
		{ code: 'ja', label: '日本語', available: false },
		{ code: 'es', label: 'Español', available: false }
	] as const;

	const localeOptions = LOCALES.filter((l) => l.available).map((l) => l.label);
	let localeSel = $state(
		Math.max(
			0,
			LOCALES.findIndex((l) => l.code === i18n.locale)
		)
	);
	$effect(() => {
		const l = LOCALES[localeSel];
		if (l?.available && l.code !== i18n.locale) i18n.set(l.code as Locale);
	});

	// 手机端声明区折叠（单选，默认全收起）
	type LegalKey = 'privacy' | 'disclaimer' | 'contribution';
	let openLegal = $state<LegalKey | null>(null);
</script>

{#snippet privacyBody()}
	<div class="space-y-2">
		<div class="flex gap-2">
			<EyeOff class="mt-0.5 h-4 w-4 shrink-0 text-gray-400" />
			<div class="min-w-0">
				<h4 class="text-xs font-medium text-white">{t('about.zeroDataTitle')}</h4>
				<p class="mt-0.5 text-xs leading-relaxed text-gray-400">
					{@html t('about.zeroDataDesc')}
				</p>
			</div>
		</div>

		<div class="flex gap-2">
			<Info class="mt-0.5 h-4 w-4 shrink-0 text-gray-400" />
			<div class="min-w-0">
				<h4 class="text-xs font-medium text-white">{t('about.localTitle')}</h4>
				<p class="mt-0.5 text-xs leading-relaxed text-gray-400">{@html t('about.localDesc')}</p>
			</div>
		</div>
	</div>
{/snippet}

{#snippet disclaimerBody()}
	<div class="space-y-1.5 text-xs leading-relaxed font-light text-gray-400">
		<p>{@html t('about.disclaimer1')}</p>
		<p>{@html t('about.disclaimer2')}</p>
		<p class="text-amber-500/80 italic">{@html t('about.disclaimer3')}</p>
	</div>
{/snippet}

{#snippet contributionBody()}
	<div class="space-y-1.5 text-xs leading-relaxed font-light text-gray-400">
		<p>{@html t('about.contribution1')}</p>
		<p>{@html t('about.contribution2')}</p>
	</div>
{/snippet}

<div class="relative mx-auto flex h-full w-full max-w-3xl flex-col text-gray-200 select-none">
	<!-- 头部标题 -->
	<header class="flex shrink-0 items-center gap-3 border-b border-gray-800 p-6">
		<img src={favicon} alt="PaintBox" class="h-10 w-10 shrink-0 object-contain" />
		<div class="min-w-0">
			<h1 class="text-lg font-bold tracking-wide text-white">PaintBox</h1>
			<p class="mt-0.5 flex flex-wrap items-center gap-x-2 font-mono text-xs text-gray-400">
				<span>Version {__APP_VERSION__}</span>
				{#if isTauri() && updater}
					<button
						type="button"
						onclick={() => updater!.check()}
						disabled={updater.state.status === 'checking'}
						class="cursor-pointer rounded border border-gray-700 px-1.5 py-0.5 text-gray-300 transition-colors hover:bg-gray-700 hover:text-white disabled:cursor-default disabled:opacity-50"
					>
						{updater.state.status === 'checking' ? t('about.checking') : t('about.checkUpdate')}
					</button>
					{#if updater.state.status === 'up-to-date'}
						<span class="text-gray-500">{t('about.upToDate')}</span>
					{:else if updater.state.status === 'outdated'}
						<span class="text-green-400"
							>{t('about.updateAvailable', { n: updater.state.latest ?? '' })}</span
						>
						<a
							href={updater.channelUrl}
							target="_blank"
							onclick={(e) => {
								if (isTauri()) {
									e.preventDefault();
									openExternal(updater!.channelUrl);
								}
							}}
							class="inline-flex cursor-pointer items-center gap-1 rounded border border-gray-700 px-1.5 py-0.5 text-gray-300 transition-colors hover:bg-gray-700 hover:text-white"
						>
							{#if updater.channelId === 'sideload'}
								<Github class="h-3 w-3" />
							{:else}
								<PlayStore class="h-3 w-3" />
							{/if}
							{t('about.viewUpdate')}
						</a>
					{:else if updater.state.status === 'error'}
						<span class="text-gray-500">
							{t('about.checkFailed')}
							{#if updater.state.error}
								<span class="text-gray-600" title={updater.state.error}
									>({updater.state.error})</span
								>
							{/if}
						</span>
					{/if}
				{/if}
			</p>
		</div>
	</header>

	<!-- 可滚动内容区：设置 + 隐私与法律 两个同级分组 -->
	<div class="min-h-0 flex-1 overflow-y-auto px-6 py-4">
		<div class="flex flex-col gap-5 sm:flex-row">
			<!-- 1. 设置 -->
			<section class="sm:w-72 sm:shrink-0">
				<div class="mb-2 flex items-center gap-2 text-primary-400">
					<SettingsIcon class="h-5 w-5" />
					<h2 class="text-lg font-semibold">{t('settings.title')}</h2>
				</div>

				<div class="divide-y divide-gray-800">
					<!-- 语言 -->
					<div class="flex items-center justify-between gap-3 py-2.5">
						<div class="min-w-0">
							<h3 class="text-sm font-medium text-white">{t('settings.lang')}</h3>
							<p class="text-[10px] text-gray-400">{t('settings.langDesc')}</p>
						</div>
						<Select class="w-32" options={localeOptions} bind:value={localeSel} />
					</div>

					<!-- 源语言油漆名称 -->
					<div class="flex items-center justify-between gap-3 py-2.5">
						<div class="min-w-0">
							<h3 class="text-sm font-medium text-white">{t('settings.displayRaw')}</h3>
							<p class="text-[10px] text-gray-400">{t('settings.displayRawDesc')}</p>
						</div>
						<Toggle
							checked={settings.displayRaw}
							onchange={(e) => settings.setDisplayRaw(e.currentTarget.checked)}
							classes={{ span: 'me-0!' }}
						/>
					</div>

					<!-- 深色模式（预留） -->
					<div class="flex items-center justify-between gap-3 py-2.5">
						<div class="min-w-0">
							<h3 class="text-sm font-medium text-white">{t('settings.theme')}</h3>
							<p class="text-[10px] text-gray-400">{t('settings.themeComingSoon')}</p>
						</div>
						<Button
							color="alternative"
							disabled
							title={t('settings.themeComingSoon')}
							class="cursor-not-allowed! bg-gray-700/60!"
						>
							<Moon class="h-4 w-4" />
						</Button>
					</div>
				</div>
			</section>

			<!-- 2. 隐私与法律（手机折叠 / sm+ 侧边直接显示） -->
			<div class="min-w-0 flex-1">
				<div class="mb-2 flex items-center gap-2 text-primary-400">
					<Info class="h-5 w-5" />
					<h2 class="text-lg font-semibold">{t('settings.legal')}</h2>
				</div>

				{#if !isMedia().sm}
					<!-- 手机：折叠（默认收起） -->
					<div class="space-y-2">
						{#snippet legalFold(
							key: LegalKey,
							title: string,
							icon: typeof ShieldCheck,
							toneCls: string
						)}
							{@const Icon = icon}
							{@const open = openLegal === key}
							<div
								class="overflow-hidden rounded-lg border transition-colors {open
									? toneCls
									: 'border-gray-800 bg-gray-800/20'}"
							>
								<button
									type="button"
									onclick={() => (openLegal = open ? null : key)}
									aria-expanded={open}
									class="flex w-full cursor-pointer items-center gap-1.5 px-3 py-2 text-left text-xs font-semibold"
								>
									<Icon class="h-4 w-4" />
									<span>{title}</span>
									<ChevronDown
										class="ms-auto h-4 w-4 shrink-0 transition-transform {open ? 'rotate-180' : ''}"
									/>
								</button>
								{#if open}
									<div class="px-3 pb-3" transition:slide={{ duration: 150 }}>
										{#if key === 'privacy'}
											{@render privacyBody()}
										{:else if key === 'disclaimer'}
											{@render disclaimerBody()}
										{:else}
											{@render contributionBody()}
										{/if}
									</div>
								{/if}
							</div>
						{/snippet}

						{@render legalFold(
							'privacy',
							t('about.privacyTitle'),
							ShieldCheck,
							'border-primary-900 bg-primary-950/20'
						)}
						{@render legalFold(
							'disclaimer',
							t('about.disclaimerTitle'),
							TriangleAlert,
							'border-amber-900 bg-amber-950/20'
						)}
						{@render legalFold(
							'contribution',
							t('about.contributionTitle'),
							Handshake,
							'border-teal-900 bg-teal-950/20'
						)}
					</div>
				{:else}
					<div class="space-y-3">
						<div class="rounded-lg border border-primary-900 bg-primary-950/20 p-3">
							<div class="flex items-center gap-1.5 text-primary-400">
								<ShieldCheck class="h-4 w-4" />
								<h3 class="text-xs font-semibold">{t('about.privacyTitle')}</h3>
							</div>
							<div class="mt-2">{@render privacyBody()}</div>
						</div>

						<div class="rounded-lg border border-amber-900 bg-amber-950/20 p-3">
							<div class="flex items-center gap-1.5 text-amber-500">
								<TriangleAlert class="h-4 w-4" />
								<h3 class="text-xs font-semibold">{t('about.disclaimerTitle')}</h3>
							</div>
							<div class="mt-2">{@render disclaimerBody()}</div>
						</div>

						<div class="rounded-lg border border-teal-900 bg-teal-950/20 p-3">
							<div class="flex items-center gap-1.5 text-teal-400">
								<Handshake class="h-4 w-4" />
								<h3 class="text-xs font-semibold">{t('about.contributionTitle')}</h3>
							</div>
							<div class="mt-2">{@render contributionBody()}</div>
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- 3. 链接与反馈（固定显示） -->
	<footer class="shrink-0 border-t border-gray-800 px-6 py-3">
		<div class="flex flex-row items-center justify-center gap-3">
			<a
				class="cursor-pointer text-white"
				href={githubUrl}
				target="_blank"
				onclick={(e) => {
					if (isTauri()) {
						e.preventDefault();
						openExternal(githubUrl);
					}
				}}
			>
				<Github class="h-7 w-7" />
			</a>
			<a
				class="inline-flex cursor-pointer items-center justify-center rounded-full bg-[#5865F2] p-1.5"
				href={discordUrl}
				target="_blank"
				onclick={(e) => {
					if (isTauri()) {
						e.preventDefault();
						openExternal(discordUrl);
					}
				}}
			>
				<Discord class="h-4.5 w-4.5 text-white" />
			</a>
			<a
				class="inline-flex cursor-pointer items-center justify-center rounded-full bg-[#2365da] p-1.5"
				href={qqUrl}
				target="_blank"
				onclick={(e) => {
					if (isTauri()) {
						e.preventDefault();
						openExternal(qqUrl);
					}
				}}
			>
				<Qq class="h-4.5 w-4.5 text-white" />
			</a>
			<a
				href={donateUrl}
				target="_blank"
				rel="noopener noreferrer"
				onclick={(e) => {
					if (isTauri()) {
						e.preventDefault();
						openExternal(donateUrl);
					}
				}}
				class="inline-flex items-center gap-1.5 rounded-full bg-yellow-400 p-1.5 text-xs font-bold text-gray-900 shadow-lg shadow-yellow-400/20 transition-colors hover:bg-yellow-300"
			>
				<Coffee class="h-3.5 w-3.5" strokeWidth={2.5} />
				{t('about.buyMeCoffee')}
			</a>
		</div>
	</footer>
</div>
