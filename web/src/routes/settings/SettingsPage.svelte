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
		Sun,
		ChevronDown,
		Settings as SettingsIcon,
		MonitorCog,
		Database,
		Download,
		Upload
	} from '@lucide/svelte';
	import { Button, ButtonGroup, Toggle } from 'flowbite-svelte';
	import { slide } from 'svelte/transition';
	import favicon from '$lib/assets/favicon.svg';
	import { i18n, t, type Locale } from '$lib/i18n.svelte';
	import { settings } from '$lib/settings.svelte';
	import { updater } from '$lib/update.svelte';
	import Select from '$lib/components/Select.svelte';
	import { isTauri } from '@tauri-apps/api/core';
	import { isMedia, openExternal } from '$lib/utils.svelte';
	import {
		exportBackup,
		parseBackup,
		applyBackup,
		type ImportMode,
		type ImportScope
	} from '$lib/backup.svelte';
	import { save, open } from '@tauri-apps/plugin-dialog';
	import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs';
	import { Radio } from 'flowbite-svelte';

	const githubUrl = 'https://github.com/cos-y/paintbox';
	const discordUrl = 'https://discord.gg/QG2ZdVRkxN'; // TODO: 替换为实际邀请链接
	const qqUrl = isTauri()
		? 'mqqapi://card/show_pslcard?src_type=internal&version=1&uin=963504621'
		: 'https://qm.qq.com/q/3bWtHScQUo';
	const donateUrl =
		'https://checkout.dodopayments.com/buy/pdt_0NkjxzuTH5BrL86pYuKnT?quantity=1&minimalAddress=true&fullName=[Anonymous]';

	// 安卓样式数字版本号：minor*1000 + patch（0.3.7 → 3007），与 tauri.properties versionCode 对齐
	const VERSION_PARTS = __APP_VERSION__.split('.').map(Number);
	const versionCode = (VERSION_PARTS[1] ?? 0) * 1000 + (VERSION_PARTS[2] ?? 0);

	// 语言选项：所有语言字典已就绪
	const LOCALES = [
		{ code: 'en', label: 'English', available: true },
		{ code: 'zh', label: '中文', available: true },
		{ code: 'ja', label: '日本語', available: true },
		{ code: 'es', label: 'Español', available: true }
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

	// ---- 数据备份 / 恢复 ----
	let importMode = $state<ImportMode>('merge');
	let importScope = $state<ImportScope>('both');
	let importMsg = $state<{ kind: 'ok' | 'error'; text: string } | null>(null);
	let exportMsg = $state<string | null>(null);
	let fileInput = $state<HTMLInputElement | undefined>(undefined);

	const stamp = () => {
		const d = new Date();
		const p = (n: number) => String(n).padStart(2, '0');
		return `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}`;
	};

	const downloadBackup = async () => {
		const json = exportBackup();
		const fname = `paintbox-backup-${stamp()}.json`;
		try {
			if (isTauri()) {
				const path = await save({
					title: t('data.downloadBackup'),
					defaultPath: fname,
					filters: [{ name: 'JSON', extensions: ['json'] }]
				});
				if (!path) return; // 用户取消
				await writeTextFile(path, json);
				exportMsg = t('data.exportDone');
			} else {
				const blob = new Blob([json], { type: 'application/json' });
				const url = URL.createObjectURL(blob);
				const a = document.createElement('a');
				a.href = url;
				a.download = fname;
				a.click();
				URL.revokeObjectURL(url);
			}
		} catch {
			exportMsg = t('data.readFailed');
		}
	};

	const pickFile = () =>
		new Promise<File | null>((resolve) => {
			const input = fileInput;
			if (!input) return resolve(null);
			input.onchange = () => {
				const f = input.files?.[0] ?? null;
				input.value = '';
				resolve(f);
			};
			input.click();
		});

	const importBackup = async () => {
		let text: string;
		try {
			if (isTauri()) {
				const path = await open({
					multiple: false,
					directory: false,
					filters: [{ name: 'JSON', extensions: ['json'] }]
				});
				if (!path) return; // 用户取消
				text = await readTextFile(path as string);
			} else {
				const file = await pickFile();
				if (!file) return; // 用户取消
				text = await file.text();
			}
		} catch {
			importMsg = { kind: 'error', text: t('data.readFailed') };
			return;
		}
		const res = parseBackup(text);
		if (!res.ok || !res.data) {
			importMsg = {
				kind: 'error',
				text: t(res.error === 'invalidJson' ? 'data.invalidJson' : 'data.invalidSchema')
			};
			return;
		}
		const r = applyBackup(res.data, importScope, importMode);
		importMsg = { kind: 'ok', text: t('data.importDone', { added: r.added, total: r.total }) };
	};
</script>

{#snippet privacyBody()}
	<div class="space-y-2">
		<div class="flex gap-2">
			<EyeOff class="mt-0.5 h-4 w-4 shrink-0 text-gray-500 dark:text-gray-400" />
			<div class="min-w-0">
				<h4 class="text-xs font-medium text-gray-900 dark:text-white">
					{t('about.zeroDataTitle')}
				</h4>
				<p class="mt-0.5 text-xs leading-relaxed text-gray-500 dark:text-gray-400">
					{@html t('about.zeroDataDesc')}
				</p>
			</div>
		</div>

		<div class="flex gap-2">
			<Info class="mt-0.5 h-4 w-4 shrink-0 text-gray-500 dark:text-gray-400" />
			<div class="min-w-0">
				<h4 class="text-xs font-medium text-gray-900 dark:text-white">{t('about.localTitle')}</h4>
				<p class="mt-0.5 text-xs leading-relaxed text-gray-500 dark:text-gray-400">
					{@html t('about.localDesc')}
				</p>
			</div>
		</div>
	</div>
{/snippet}

{#snippet disclaimerBody()}
	<div class="space-y-1.5 text-xs leading-relaxed font-light text-gray-500 dark:text-gray-400">
		<p>{@html t('about.disclaimer1')}</p>
		<p>{@html t('about.disclaimer2')}</p>
		<p class="text-amber-600/80 italic dark:text-amber-500/80">{@html t('about.disclaimer3')}</p>
	</div>
{/snippet}

{#snippet contributionBody()}
	<div class="space-y-1.5 text-xs leading-relaxed font-light text-gray-500 dark:text-gray-400">
		<p>{@html t('about.contribution1')}</p>
		<p>{@html t('about.contribution2')}</p>
	</div>
{/snippet}

<div
	class="relative mx-auto flex h-full w-full max-w-3xl flex-col text-gray-900 select-none dark:text-gray-200"
>
	<input type="file" accept=".json,application/json" bind:this={fileInput} class="hidden" />
	<!-- 头部标题 -->
	<header class="border-theme mx-6 flex shrink-0 items-center gap-3 border-b py-6">
		<img src={favicon} alt="PaintBox" class="h-10 w-10 shrink-0 object-contain" />
		<div class="min-w-0">
			<h1 class="text-lg font-bold tracking-wide text-gray-900 dark:text-white">PaintBox</h1>
			<p
				class="mt-0.5 flex flex-wrap items-center gap-x-2 font-mono text-xs text-gray-500 dark:text-gray-400"
			>
				<span>Version {__APP_VERSION__} ({versionCode})</span>
				{#if isTauri() && updater}
					<button
						type="button"
						onclick={() => updater!.check()}
						disabled={updater.state.status === 'checking'}
						class="border-theme cursor-pointer rounded border px-1.5 py-0.5 text-gray-600 transition-colors hover:bg-gray-200 hover:text-gray-900 disabled:cursor-default disabled:opacity-50 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white"
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
							class="border-theme inline-flex cursor-pointer items-center gap-1 rounded border px-1.5 py-0.5 text-gray-600 transition-colors hover:bg-gray-200 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white"
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
			<!-- 左列：设置 + 数据 -->
		<div class="flex flex-col gap-5 sm:w-72 sm:shrink-0">
			<!-- 1. 设置 -->
			<section>
				<div class="mb-2 flex items-center gap-2 text-primary-600 dark:text-primary-400">
					<SettingsIcon class="h-5 w-5" />
					<h2 class="text-lg font-semibold">{t('settings.title')}</h2>
				</div>

				<div class="divide-y divide-gray-200 dark:divide-gray-800">
					<!-- 语言 -->
					<div class="flex items-center justify-between gap-3 py-2.5">
						<div class="min-w-0">
							<h3 class="text-sm font-medium text-gray-900 dark:text-white">
								{t('settings.lang')}
							</h3>
							<p class="text-[10px] text-gray-500 dark:text-gray-400">{t('settings.langDesc')}</p>
						</div>
						<Select class="w-32" options={localeOptions} bind:value={localeSel} />
					</div>

					<!-- 源语言油漆名称 -->
					<div class="flex items-center justify-between gap-3 py-2.5">
						<div class="min-w-0">
							<h3 class="text-sm font-medium text-gray-900 dark:text-white">
								{t('settings.displayRaw')}
							</h3>
							<p class="text-[10px] text-gray-500 dark:text-gray-400">
								{t('settings.displayRawDesc')}
							</p>
						</div>
						<Toggle
							checked={settings.displayRaw}
							onchange={(e) => settings.setDisplayRaw(e.currentTarget.checked)}
							class="cursor-pointer"
							classes={{ span: 'me-0! peer-checked:bg-primary-500!' }}
						/>
					</div>

					<!-- 主题 -->
					<div class="flex items-center justify-between gap-3 py-2.5">
						<div class="min-w-0">
							<h3 class="text-sm font-medium text-gray-900 dark:text-white">
								{t('settings.theme')}
							</h3>
							<p class="text-[10px] text-gray-500 dark:text-gray-400">{t('settings.themeDesc')}</p>
						</div>
						<ButtonGroup size="sm" class="shrink-0">
							<Button
								onclick={() => settings.setTheme('system')}
								class="cursor-pointer! {settings.theme === 'system'
									? 'bg-primary-500! text-white!'
									: ''}"
								title={t('settings.themeSystem')}
							>
								<MonitorCog class="size-3.5" />
							</Button>
							<Button
								onclick={() => settings.setTheme('light')}
								class="cursor-pointer! {settings.theme === 'light'
									? 'bg-primary-500! text-white!'
									: ''}"
								title={t('settings.themeLight')}
							>
								<Sun class="size-3.5" />
							</Button>
							<Button
								onclick={() => settings.setTheme('dark')}
								class="cursor-pointer! {settings.theme === 'dark'
									? 'bg-primary-500! text-white!'
									: ''}"
								title={t('settings.themeDark')}
							>
								<Moon class="size-3.5" />
							</Button>
						</ButtonGroup>
					</div>

				</div>
			</section>

			<!-- 2. 数据备份 / 恢复 -->
			<section>
				<div class="mb-2 flex items-center gap-2 text-primary-600 dark:text-primary-400">
					<Database class="h-5 w-5" />
					<h2 class="text-lg font-semibold">{t('data.title')}</h2>
				</div>
				<div>
					<p class="text-[10px] text-gray-500 dark:text-gray-400">
							{t('data.exportDesc')}
						</p>
						<button
							type="button"
							onclick={downloadBackup}
							class="border-theme mt-2 flex w-full cursor-pointer items-center justify-center gap-1.5 rounded-lg border px-3 py-1.5 text-sm font-medium text-gray-600 transition-colors hover:bg-gray-200 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white"
						>
							<Download class="size-4" />
							{t('data.downloadBackup')}
						</button>
						{#if exportMsg}
							<p class="mt-1.5 text-[11px] text-green-600 dark:text-green-400">{exportMsg}</p>
						{/if}

						<div class="mt-3 border-t border-gray-200 pt-3 dark:border-gray-800">
							<h4 class="text-xs font-medium text-gray-900 dark:text-white">
								{t('data.scopeTitle')}
							</h4>
							<div class="mt-1.5 space-y-1.5">
								<label class="flex cursor-pointer items-center gap-2">
									<Radio name="importScope" value="stock" bind:group={importScope} class="mt-0.5" />
									<span class="text-xs font-medium text-gray-900 dark:text-white">
										{t('data.scopeStock')}
									</span>
								</label>
								<label class="flex cursor-pointer items-center gap-2">
									<Radio name="importScope" value="both" bind:group={importScope} class="mt-0.5" />
									<span class="text-xs font-medium text-gray-900 dark:text-white">
										{t('data.scopeBoth')}
									</span>
								</label>
								<label class="flex cursor-pointer items-center gap-2">
									<Radio
										name="importScope"
										value="settings"
										bind:group={importScope}
										class="mt-0.5"
									/>
									<span class="text-xs font-medium text-gray-900 dark:text-white">
										{t('data.scopeSettings')}
									</span>
								</label>
							</div>

							{#if importScope === 'stock' || importScope === 'both'}
								<h4 class="mt-3 text-xs font-medium text-gray-900 dark:text-white">
									{t('data.importModeTitle')}
								</h4>
								<div class="mt-1.5 space-y-1.5">
									<label class="flex cursor-pointer items-start gap-2">
										<Radio name="importMode" value="merge" bind:group={importMode} class="mt-0.5" />
										<div class="min-w-0">
											<span class="text-xs font-medium text-gray-900 dark:text-white">
												{t('data.modeMerge')}
											</span>
											<p class="text-[10px] text-gray-500 dark:text-gray-400">
												{t('data.modeMergeDesc')}
											</p>
										</div>
									</label>
									<label class="flex cursor-pointer items-start gap-2">
										<Radio
											name="importMode"
											value="replace"
											bind:group={importMode}
											class="mt-0.5"
										/>
										<div class="min-w-0">
											<span class="text-xs font-medium text-red-600 dark:text-red-400">
												{t('data.modeReplace')}
											</span>
											<p class="text-[10px] text-gray-500 dark:text-gray-400">
												{t('data.modeReplaceDesc')}
											</p>
										</div>
									</label>
								</div>
							{/if}
							<button
								type="button"
								class="border-theme mt-2 flex w-full cursor-pointer items-center justify-center gap-1.5 rounded-lg border px-3 py-1.5 text-sm font-medium text-gray-600 transition-colors hover:bg-gray-200 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white"
								onclick={importBackup}
							>
								<Upload class="size-4" />
								{t('data.importButton')}
							</button>
							{#if importMsg}
								<p
									class="mt-1.5 text-[11px] {importMsg.kind === 'ok'
										? 'text-green-600 dark:text-green-400'
										: 'text-red-600 dark:text-red-400'}"
								>
									{importMsg.text}
								</p>
							{/if}
						</div>
				</div>
			</section>
		</div>

			<!-- 2. 隐私与法律（手机折叠 / sm+ 侧边直接显示） -->
			<div class="min-w-0 flex-1">
				<div class="mb-2 flex items-center gap-2 text-primary-600 dark:text-primary-400">
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
									: 'border-theme bg-gray-50 dark:bg-gray-800/20'}"
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
							'border-primary-200 bg-primary-50 dark:border-primary-900 dark:bg-primary-950/20'
						)}
						{@render legalFold(
							'disclaimer',
							t('about.disclaimerTitle'),
							TriangleAlert,
							'border-amber-200 bg-amber-50 dark:border-amber-900 dark:bg-amber-950/20'
						)}
						{@render legalFold(
							'contribution',
							t('about.contributionTitle'),
							Handshake,
							'border-teal-200 bg-teal-50 dark:border-teal-900 dark:bg-teal-950/20'
						)}
					</div>
				{:else}
					<div class="space-y-3">
						<div
							class="rounded-lg border border-primary-200 bg-primary-50 p-3 dark:border-primary-900 dark:bg-primary-950/20"
						>
							<div class="flex items-center gap-1.5 text-primary-600 dark:text-primary-400">
								<ShieldCheck class="h-4 w-4" />
								<h3 class="text-xs font-semibold">{t('about.privacyTitle')}</h3>
							</div>
							<div class="mt-2">{@render privacyBody()}</div>
						</div>

						<div
							class="rounded-lg border border-amber-200 bg-amber-50 p-3 dark:border-amber-900 dark:bg-amber-950/20"
						>
							<div class="flex items-center gap-1.5 text-amber-600 dark:text-amber-500">
								<TriangleAlert class="h-4 w-4" />
								<h3 class="text-xs font-semibold">{t('about.disclaimerTitle')}</h3>
							</div>
							<div class="mt-2">{@render disclaimerBody()}</div>
						</div>

						<div
							class="rounded-lg border border-teal-200 bg-teal-50 p-3 dark:border-teal-900 dark:bg-teal-950/20"
						>
							<div class="flex items-center gap-1.5 text-teal-600 dark:text-teal-400">
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
	<footer class="border-theme mx-6 shrink-0 border-t py-3">
		<div class="flex flex-row items-center justify-center gap-3">
			<a
				class="cursor-pointer text-gray-900 dark:text-white"
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
