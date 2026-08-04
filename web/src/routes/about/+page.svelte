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
		Languages
	} from '@lucide/svelte';
	import { isTauri } from '$lib/utils';
	import favicon from '$lib/assets/favicon.svg';
	import { i18n, toggleLocale, t } from '$lib/i18n.svelte';
	import { updateChecker, channelUrl, channelId } from '$lib/update.svelte';
	// Tauri 环境显式调 opener 插件打开外链（绕开 WebView 对自定义 scheme 的解析）
	import { openUrl } from '@tauri-apps/plugin-opener';

	const githubUrl = 'https://github.com/cos-y/paintbox';
	const discordUrl = 'https://discord.gg/QG2ZdVRkxN'; // TODO: 替换为实际邀请链接
	const qqUrl = isTauri
		? 'mqqapi://card/show_pslcard?src_type=internal&version=1&uin=963504621'
		: 'https://qm.qq.com/q/3bWtHScQUo';
	const donateUrl = 'https://afdian.com/a/cos_y';

	// Tauri 内用系统 Intent 打开（QQ 链接可直接唤起 QQ App）；浏览器内普通新标签
	const openExternal = (url: string) => {
		if (isTauri) openUrl(url);
		else window.open(url, '_blank', 'noopener');
	};
</script>

<div class="mx-auto flex h-full w-full max-w-3xl flex-col p-6 text-gray-200 select-none">
	<!-- 可滚动内容区 -->
	<div class="min-h-0 flex-1 space-y-4 overflow-y-auto pr-1">
		<!-- 头部标题 -->
		<div class="flex items-center gap-4 border-b border-gray-800 pb-4">
			<img src={favicon} alt="PaintBox" class="h-16 w-16 shrink-0 object-contain" />
			<div class="min-w-0">
				<h1 class="text-2xl font-bold tracking-wide text-white">PaintBox</h1>
				<p class="mt-1 flex flex-wrap items-center gap-x-2 font-mono text-xs text-gray-400">
					<span>Version {__APP_VERSION__}</span>
					{#if isTauri}
						<button
							type="button"
							onclick={() => updateChecker.check()}
							disabled={updateChecker.state.status === 'checking'}
							class="cursor-pointer rounded border border-gray-700 px-1.5 py-0.5 text-gray-300 transition-colors hover:bg-gray-700 hover:text-white disabled:cursor-default disabled:opacity-50"
						>
							{updateChecker.state.status === 'checking'
								? t('about.checking')
								: t('about.checkUpdate')}
						</button>
						{#if updateChecker.state.status === 'up-to-date'}
							<span class="text-gray-500">{t('about.upToDate')}</span>
						{:else if updateChecker.state.status === 'outdated'}
							<span class="text-green-400"
								>{t('about.updateAvailable', { n: updateChecker.state.latest ?? '' })}</span
							>
							<a
								href={channelUrl}
								target="_blank"
								onclick={(e) => {
									if (isTauri) {
										e.preventDefault();
										openExternal(channelUrl);
									}
								}}
								class="inline-flex cursor-pointer items-center gap-1 rounded border border-gray-700 px-1.5 py-0.5 text-gray-300 transition-colors hover:bg-gray-700 hover:text-white"
							>
								{#if channelId === 'sideload'}
									<Github class="h-3 w-3" />
								{:else}
									<PlayStore class="h-3 w-3" />
								{/if}
								{t('about.viewUpdate')}
							</a>
						{:else if updateChecker.state.status === 'error'}
							<span class="text-gray-500">{t('about.checkFailed')}</span>
						{/if}
					{/if}
				</p>
			</div>
			<!-- 语言切换（后续设置区入口） -->
			<div class="ml-auto shrink-0">
				<button
					type="button"
					onclick={toggleLocale}
					title={i18n.locale === 'en' ? t('about.switchToZh') : t('about.switchToEn')}
					class="flex cursor-pointer items-center gap-1.5 rounded-md border border-gray-700 bg-gray-800/60 px-2.5 py-1.5 text-xs text-gray-300 transition-colors hover:bg-gray-700 hover:text-white"
				>
					<Languages class="h-3.5 w-3.5" />
					{i18n.locale === 'en' ? '中文' : 'EN'}
				</button>
			</div>
		</div>

		<!-- 1. 隐私与离线声明 -->
		<section class="space-y-4">
			<div class="flex items-center gap-2 text-primary-400">
				<ShieldCheck class="h-5 w-5" />
				<h2 class="text-base font-semibold">{t('about.privacyTitle')}</h2>
			</div>

			<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
				<div class="flex gap-3 rounded-lg border border-gray-500 bg-gray-800/40 p-4">
					<EyeOff class="mt-0.5 h-5 w-5 shrink-0 text-gray-400" />
					<div>
						<h3 class="text-sm font-medium text-white">{t('about.zeroDataTitle')}</h3>
						<p class="mt-1 text-xs leading-relaxed text-gray-400">
							{@html t('about.zeroDataDesc')}
						</p>
					</div>
				</div>

				<div class="flex gap-3 rounded-lg border border-gray-500 bg-gray-800/40 p-4">
					<Info class="mt-0.5 h-5 w-5 shrink-0 text-gray-400" />
					<div>
						<h3 class="text-sm font-medium text-white">{t('about.localTitle')}</h3>
						<p class="mt-1 text-xs leading-relaxed text-gray-400">
							{@html t('about.localDesc')}
						</p>
					</div>
				</div>
			</div>
		</section>

		<!-- 2. 免责声明 -->
		<section class="space-y-3 rounded-lg border border-amber-900 bg-amber-950/20 p-4">
			<div class="flex items-center gap-2 text-amber-500">
				<TriangleAlert class="h-5 w-5" />
				<h2 class="text-sm font-semibold">{t('about.disclaimerTitle')}</h2>
			</div>
			<div class="space-y-2 text-xs leading-relaxed font-light text-gray-400">
				<p>{@html t('about.disclaimer1')}</p>
				<p>{@html t('about.disclaimer2')}</p>
				<p class="text-amber-500/80 italic">{@html t('about.disclaimer3')}</p>
			</div>
		</section>

		<!-- 3. 数据贡献声明 -->
		<section class="space-y-3 rounded-lg border border-teal-900 bg-teal-950/20 p-4">
			<div class="flex items-center gap-2 text-teal-400">
				<Handshake class="h-5 w-5" />
				<h2 class="text-sm font-semibold">{t('about.contributionTitle')}</h2>
			</div>
			<div class="space-y-2 text-xs leading-relaxed font-light text-gray-400">
				<p>{@html t('about.contribution1')}</p>
				<p>{@html t('about.contribution2')}</p>
			</div>
		</section>
	</div>

	<!-- 4. 链接与反馈（固定显示） -->
	<section class="mt-4 shrink-0 border-t border-gray-800 pt-4">
		<div class="flex flex-row items-center justify-center gap-4">
			<a
				class="cursor-pointer text-white"
				href={githubUrl}
				target="_blank"
				onclick={(e) => {
					if (isTauri) {
						e.preventDefault();
						openExternal(githubUrl);
					}
				}}
			>
				<Github class="h-8 w-8" />
			</a>
			<a
				class="inline-flex cursor-pointer items-center justify-center rounded-full bg-[#5865F2] p-1.5"
				href={discordUrl}
				target="_blank"
				onclick={(e) => {
					if (isTauri) {
						e.preventDefault();
						openExternal(discordUrl);
					}
				}}
			>
				<Discord class="h-5 w-5 text-white" />
			</a>
			<a
				class="inline-flex cursor-pointer items-center justify-center rounded-full bg-[#2365da] p-1.5"
				href={qqUrl}
				target="_blank"
				onclick={(e) => {
					if (isTauri) {
						e.preventDefault();
						openExternal(qqUrl);
					}
				}}
			>
				<Qq class="h-5 w-5 text-white" />
			</a>
			<a
				href={donateUrl}
				target="_blank"
				rel="noopener noreferrer"
				onclick={(e) => {
					if (isTauri) {
						e.preventDefault();
						openExternal(donateUrl);
					}
				}}
				class="inline-flex items-center gap-2 rounded-full bg-yellow-400 p-2 text-sm font-bold text-gray-900 shadow-lg shadow-yellow-400/20 transition-colors hover:bg-yellow-300"
			>
				<Coffee class="h-4 w-4" strokeWidth={2.5} />
				{t('about.buyMeCoffee')}
			</a>
		</div>
	</section>
</div>
