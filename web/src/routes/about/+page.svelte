<script lang="ts">
	import Github from '$lib/icons/Github.svelte';
	import { ShieldCheck, EyeOff, Info, TriangleAlert, Mail } from 'lucide-svelte';

	const githubUrl = 'https://github.com/cos-y/paintbox';
</script>

<div class="max-w-3xl mx-auto p-6 text-gray-200 space-y-4 select-none flex flex-col h-full w-full">
	<!-- 头部标题 -->
	<div class="border-b border-gray-800 pb-4">
		<h1 class="text-2xl font-bold tracking-wide text-white">About</h1>
		<p class="text-xs text-gray-400 mt-1 font-mono">Version 1.0.0 (WASM Build)</p>
	</div>

	<!-- 1. 隐私与离线声明 -->
	<section class="space-y-4">
		<div class="flex items-center gap-2 text-primary-400">
			<ShieldCheck class="h-5 w-5" />
			<h2 class="text-base font-semibold">隐私与安全声明</h2>
		</div>

		<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
			<div class="bg-gray-800/40 border border-gray-500 rounded-lg p-4 flex gap-3">
				<EyeOff class="h-5 w-5 text-gray-400 shrink-0 mt-0.5" />
				<div>
					<h3 class="text-sm font-medium text-white">零个人数据收集</h3>
					<p class="text-xs text-gray-400 mt-1 leading-relaxed">
						我们<strong class="text-gray-300 font-medium">不会收集、上传或存储</strong
						>您的任何个人数据、库存配置或检索历史。您的隐私绝对安全。
					</p>
				</div>
			</div>

			<div class="bg-gray-800/40 border border-gray-500 rounded-lg p-4 flex gap-3">
				<Info class="h-5 w-5 text-gray-400 shrink-0 mt-0.5" />
				<div>
					<h3 class="text-sm font-medium text-white">100% 本地离线运行</h3>
					<p class="text-xs text-gray-400 mt-1 leading-relaxed">
						核心调色算法与漆号检索完全在您的浏览器本地（WebAssembly）编译执行，无需联网即可稳定使用。
					</p>
				</div>
			</div>
		</div>
	</section>

	<!-- 2. 免责声明 -->
	<section class="bg-amber-950/20 border border-amber-900 rounded-lg p-4 space-y-3">
		<div class="flex items-center gap-2 text-amber-500">
			<TriangleAlert class="h-5 w-5" />
			<h2 class="text-sm font-semibold">混色算法免责声明</h2>
		</div>
		<div class="text-xs text-gray-400 space-y-2 leading-relaxed font-light">
			<p>
				1. 本工具所有的混色配方与色差计算结果均基于<a
					class="underline"
					href="https://scrtwpns.com/mixbox.pdf"
					target="_blank"
					rel="noopener noreferrer"
				>
					色彩科学理论
				</a>模拟得出，<strong class="text-gray-300 font-medium">实际发色效果仅供参考</strong>。
			</p>
			<p>
				2. 由于不同厂商以及不同系列油漆的化学性质、颜料颗粒密度及遮盖力存在物理差异，<strong
					class="text-gray-300 font-medium">具体混色可行性请务必结合厂商提供的涂料性质决定</strong
				>。
			</p>
			<p class="text-amber-500/80 italic">
				* 提示：在大面积喷涂前，请务必进行局部试喷以验证实际发色。
			</p>
		</div>
	</section>

	<!-- 3. 链接与反馈 -->
	<section class="mt-auto pt-4 border-t border-gray-800">
		<div class="flex flex-row justify-center gap-4">
			<a class="cursor-pointer text-white" href={githubUrl} target="_blank">
				<Github class="w-8 h-8" />
			</a>
		</div>
	</section>
</div>
