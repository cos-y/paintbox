<script lang="ts">
	import { Camera, CameraOff } from '@lucide/svelte';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		/** 拍照取色回调，参数为 sRGB 0-1 浮点值 */
		onsample: (r: number, g: number, b: number) => void;
	}

	let { onsample }: Props = $props();

	let video: HTMLVideoElement | undefined = $state();
	let canvas: HTMLCanvasElement | undefined = $state();
	let error = $state(false);
	let ready = $state(false);
	// 中心像素实时颜色
	let live: [number, number, number] = $state([0, 0, 0]);

	const liveHex = $derived(
		'#' +
			live
				.map((v) =>
					Math.round(v * 255)
						.toString(16)
						.padStart(2, '0')
				)
				.join('')
	);

	// 启动摄像头（环境摄像头，组件卸载时自动停止）
	$effect(() => {
		const v = video;
		if (!v) return;
		let cancelled = false;
		(async () => {
			try {
				const stream = await navigator.mediaDevices.getUserMedia({
					video: { facingMode: 'environment' }
				});
				if (cancelled) {
					stream.getTracks().forEach((t) => t.stop());
					return;
				}
				v.srcObject = stream;
				await v.play();
				ready = true;
			} catch {
				if (!cancelled) {
					error = true;
				}
			}
		})();
		return () => {
			cancelled = true;
			const s = v.srcObject as MediaStream | null;
			s?.getTracks().forEach((t) => t.stop());
			v.srcObject = null;
		};
	});

	// 定时采样画面正中心像素（object-fit: cover 下显示中心 = 视频中心）
	const centerSample = () => {
		if (!video || !ready || !canvas) return;
		const vw = video.videoWidth;
		const vh = video.videoHeight;
		if (!vw || !vh) return;
		canvas.width = vw;
		canvas.height = vh;
		const ctx = canvas.getContext('2d');
		if (!ctx) return;
		ctx.drawImage(video, 0, 0);
		const [r, g, b] = ctx.getImageData(vw >> 1, vh >> 1, 1, 1).data;
		live = [r / 255, g / 255, b / 255];
	};

	$effect(() => {
		if (!ready) return;
		const t = setInterval(centerSample, 150);
		return () => clearInterval(t);
	});

	const capture = () => {
		if (!ready) return;
		onsample(live[0], live[1], live[2]);
	};
</script>

<div class="overflow-hidden rounded-xl border border-gray-700">
	<!-- 取景框 -->
	<div class="relative aspect-video bg-black">
		{#if error}
			<div
				class="flex h-full items-center justify-center gap-2 px-4 text-center text-xs text-gray-300"
			>
				<CameraOff class="size-5" />
				<span>{t('camera.accessError')}</span>
			</div>
		{:else}
			<canvas bind:this={canvas} class="hidden"></canvas>
			<video
				bind:this={video}
				class="h-full w-full object-cover"
				autoplay
				playsinline
				muted
				onloadedmetadata={() => (ready = true)}
			>
			</video>
			{#if !ready}
				<div
					class="absolute inset-0 flex items-center justify-center bg-black text-xs text-white/75"
				>
					{t('camera.launching')}
				</div>
			{:else}
				<!-- 中心准星 -->
				<div class="pointer-events-none absolute inset-0 flex items-center justify-center">
					<div class="relative h-12 w-12">
						<div class="absolute top-0 left-1/2 h-full w-px -translate-x-1/2 bg-white/80"></div>
						<div class="absolute top-1/2 left-0 h-px w-full -translate-y-1/2 bg-white/80"></div>
						<div
							class="absolute top-1/2 left-1/2 h-2 w-2 -translate-x-1/2 -translate-y-1/2 rounded-full border border-white/80"
						></div>
					</div>
				</div>
			{/if}
		{/if}
	</div>

	<!-- 实时色值 + 拍照按钮 -->
	<div class="flex items-center gap-2 border-t border-gray-700 bg-gray-900 px-3 py-2">
		<div
			class="h-5 w-5 shrink-0 rounded-sm border border-gray-600"
			style="background: {liveHex}"
		></div>
		<span class="font-mono text-xs text-gray-300">{liveHex}</span>
		<button
			type="button"
			class="ml-auto flex cursor-pointer items-center gap-1 rounded-md bg-primary-600 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-primary-500 disabled:cursor-not-allowed disabled:opacity-50"
			title={t('camera.captureColor')}
			onclick={capture}
			disabled={!ready}
		>
			<Camera size="0.9rem" />
			{t('camera.pick')}
		</button>
	</div>
</div>
