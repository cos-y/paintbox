<script lang="ts">
	import { CameraOff } from '@lucide/svelte';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		/** 拍照取色回调，参数为 sRGB 0-1 浮点值 */
		onsample: (r: number, g: number, b: number) => void;
		/** 全屏填充模式：占满父容器（横屏摄像机全屏覆盖时使用） */
		fill?: boolean;
	}

	let { onsample, fill = false }: Props = $props();

	let video: HTMLVideoElement | undefined = $state();
	let canvas: HTMLCanvasElement | undefined = $state();
	let error = $state(false);
	let ready = $state(false);
	// 当前请求方向：竖屏为纵向流（3:4 取景框），横屏为横向流（16:9）
	let portrait = $state(true);
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

	let stream: MediaStream | null = null;
	let cancelled = true;
	let gen = 0; // 请求序号：方向切换时丢弃旧的进行中请求

	// 按当前设备方向启动/重启摄像头
	const startCamera = async () => {
		const v = video;
		if (!v || cancelled) return;
		const myGen = ++gen;
		// 停掉旧流
		if (stream) {
			stream.getTracks().forEach((t) => t.stop());
			stream = null;
		}
		ready = false;
		const p = window.matchMedia('(orientation: portrait)').matches;
		portrait = p;
		try {
			const s = await navigator.mediaDevices.getUserMedia({
				video: {
					facingMode: 'environment',
					// 按握持方向请求对应尺寸（部分设备会直接返回该方向的流）
					width: { ideal: p ? 1080 : 1920 },
					height: { ideal: p ? 1920 : 1080 }
				}
			});
			if (cancelled || myGen !== gen) {
				s.getTracks().forEach((t) => t.stop());
				return;
			}
			stream = s;
			v.srcObject = s;
			await v.play();
			ready = true;
		} catch {
			if (!cancelled) {
				error = true;
			}
		}
	};

	// 启动摄像头 + 监听屏幕方向变化（旋转时重新请求对应方向的流）
	$effect(() => {
		const v = video;
		if (!v) return;
		cancelled = false;
		const mq = window.matchMedia('(orientation: portrait)');
		const onOrient = () => {
			if (mq.matches !== portrait) startCamera();
		};
		mq.addEventListener('change', onOrient);
		startCamera();
		return () => {
			cancelled = true;
			gen++; // 丢弃进行中的请求
			mq.removeEventListener('change', onOrient);
			stream?.getTracks().forEach((t) => t.stop());
			stream = null;
			v.srcObject = null;
		};
	});

	// 定时采样画面正中心像素（显示中心 = 视频中心，cover/contain 均成立）
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

<!-- 取景框：fill 模式占满父容器；普通模式启动时横向占满（16:9），就绪后按方向切换 -->
<div
	class={fill
		? 'relative min-h-0 flex-1 bg-black'
		: `relative bg-black ${ready
				? portrait
					? 'aspect-3/4'
					: 'h-[calc(100dvh-2rem)]'
				: 'aspect-video'}`}
>
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
			<div class="absolute inset-0 flex items-center justify-center bg-black text-xs text-white/75">
				{t('camera.launching')}
			</div>
		{:else}
			<!-- 左上角：当前色卡 + 色号 -->
			<div class="absolute top-2 left-2 z-10 flex items-center gap-1.5">
				<div
					class="h-4 w-4 shrink-0 rounded-sm border border-white/60"
					style="background: {liveHex}"
				></div>
				<span class="rounded bg-black/45 px-1.5 py-0.5 font-mono text-[11px] text-white"
					>{liveHex}</span
				>
			</div>
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
			<!-- 快门：短边中点——纵屏底部居中，横屏右侧垂直居中 -->
			<div
				class="absolute {portrait
					? 'bottom-4 left-1/2 -translate-x-1/2'
					: 'top-1/2 right-4 -translate-y-1/2'}"
			>
				<button
					type="button"
					class="flex size-12 cursor-pointer items-center justify-center rounded-full border-[3px] border-white bg-black/25 shadow-lg transition-transform active:scale-90 disabled:cursor-not-allowed disabled:opacity-40"
					title={t('camera.captureColor')}
					onclick={capture}
					disabled={!ready}
				>
					<div class="size-9 rounded-full bg-white"></div>
				</button>
			</div>
		{/if}
	{/if}
</div>
