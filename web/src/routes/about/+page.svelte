<script lang="ts">
	import Github from '$lib/icons/Github.svelte';
	import Qq from '$lib/icons/Qq.svelte';
	import {
		ShieldCheck,
		EyeOff,
		Info,
		TriangleAlert,
		Mail,
		Coffee,
		Handshake
	} from '@lucide/svelte';

	const githubUrl = 'https://github.com/cos-y/paintbox';
	const qqUrl = 'https://qm.qq.com/q/3bWtHScQUo';
	const donateUrl = 'https://afdian.com/a/cos_y';
</script>

<div class="mx-auto flex h-full w-full max-w-3xl flex-col p-6 text-gray-200 select-none">
	<!-- 可滚动内容区 -->
	<div class="min-h-0 flex-1 space-y-4 overflow-y-auto pr-1">
		<!-- 头部标题 -->
		<div class="border-b border-gray-800 pb-4">
			<h1 class="text-2xl font-bold tracking-wide text-white">PaintBox</h1>
			<p class="mt-1 font-mono text-xs text-gray-400">Version {__APP_VERSION__}</p>
		</div>

		<!-- 1. 隐私与离线声明 -->
		<section class="space-y-4">
			<div class="flex items-center gap-2 text-primary-400">
				<ShieldCheck class="h-5 w-5" />
				<h2 class="text-base font-semibold">隐私与安全声明</h2>
			</div>

			<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
				<div class="flex gap-3 rounded-lg border border-gray-500 bg-gray-800/40 p-4">
					<EyeOff class="mt-0.5 h-5 w-5 shrink-0 text-gray-400" />
					<div>
						<h3 class="text-sm font-medium text-white">零个人数据收集</h3>
						<p class="mt-1 text-xs leading-relaxed text-gray-400">
							我们<strong class="font-medium text-gray-300">不会收集、上传或存储</strong
							>您的任何个人数据、库存配置或检索历史。您的隐私绝对安全。
						</p>
					</div>
				</div>

				<div class="flex gap-3 rounded-lg border border-gray-500 bg-gray-800/40 p-4">
					<Info class="mt-0.5 h-5 w-5 shrink-0 text-gray-400" />
					<div>
						<h3 class="text-sm font-medium text-white">100% 本地离线运行</h3>
						<p class="mt-1 text-xs leading-relaxed text-gray-400">
							核心调色算法与漆号检索完全在您的浏览器本地（WebAssembly）编译执行，无需联网即可稳定使用。
						</p>
					</div>
				</div>
			</div>
		</section>

		<!-- 2. 免责声明 -->
		<section class="space-y-3 rounded-lg border border-amber-900 bg-amber-950/20 p-4">
			<div class="flex items-center gap-2 text-amber-500">
				<TriangleAlert class="h-5 w-5" />
				<h2 class="text-sm font-semibold">混色算法免责声明</h2>
			</div>
			<div class="space-y-2 text-xs leading-relaxed font-light text-gray-400">
				<p>
					1. 本工具所有的混色配方与色差计算结果均基于<a
						class="underline"
						href="https://scrtwpns.com/mixbox.pdf"
						target="_blank"
						rel="noopener noreferrer"
					>
						色彩科学理论
					</a>模拟得出，<strong class="font-medium text-gray-300">实际发色效果仅供参考</strong>。
				</p>
				<p>
					2. 由于不同厂商以及不同系列油漆的化学性质、颜料颗粒密度及遮盖力存在物理差异，<strong
						class="font-medium text-gray-300">具体混色可行性请务必结合厂商提供的涂料性质决定</strong
					>。
				</p>
				<p class="text-amber-500/80 italic">
					* 提示：在大面积喷涂前，请务必进行局部试喷以验证实际发色。
				</p>
			</div>
		</section>

		<!-- 3. 数据贡献声明 -->
		<section class="space-y-3 rounded-lg border border-teal-900 bg-teal-950/20 p-4">
			<div class="flex items-center gap-2 text-teal-400">
				<Handshake class="h-5 w-5" />
				<h2 class="text-sm font-semibold">数据覆盖与贡献</h2>
			</div>
			<div class="space-y-2 text-xs leading-relaxed font-light text-gray-400">
				<p>
					部分油漆品牌因未发布<strong class="font-medium text-gray-300">官方色卡</strong
					>数据，暂未收录于本工具中，故未在此列出。
				</p>
				<p>
					如果您拥有相关品牌的数据并愿意共享，<strong class="font-medium text-gray-300"
						>欢迎贡献数据</strong
					>，可通过页面底部的 GitHub / QQ 联系我们。
				</p>
			</div>
		</section>
	</div>

	<!-- 4. 链接与反馈（固定显示） -->
	<section class="mt-4 shrink-0 border-t border-gray-800 pt-4">
		<div class="flex flex-row items-center justify-center gap-4">
			<a class="cursor-pointer text-white" href={githubUrl} target="_blank">
				<Github class="h-8 w-8" />
			</a>
			<a
				class="inline-flex cursor-pointer items-center justify-center rounded-full bg-[#2365da] p-1.5"
				href={qqUrl}
				target="_blank"
			>
				<Qq class="h-5 w-5 text-white" />
			</a>
			<a
				href={donateUrl}
				target="_blank"
				rel="noopener noreferrer"
				class="inline-flex items-center gap-2 rounded-full bg-yellow-400 p-2 text-sm font-bold text-gray-900 shadow-lg shadow-yellow-400/20 transition-colors hover:bg-yellow-300"
			>
				<Coffee class="h-4 w-4" strokeWidth={2.5} />
				Buy me a coffee
			</a>
		</div>
	</section>
</div>
