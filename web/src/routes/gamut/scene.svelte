<script lang="ts">
	import { Gizmo, OrbitControls, interactivity } from '@threlte/extras';
	import { T, useThrelte } from '@threlte/core';
	import {
		Vector3,
		Plane,
		ShaderMaterial,
		LineBasicMaterial,
		InstancedMesh,
		PerspectiveCamera,
		NoToneMapping
	} from 'three';
	import { clamp, linearToSrgb } from '$lib/utils.svelte';
	import { sceneProps } from './gamut.svelte';

	const { toneMapping, invalidate, renderer } = useThrelte();
	toneMapping.set(NoToneMapping);
	renderer.localClippingEnabled = true;

	interface Props {
		ndiv: number;
		matrices: Float32Array;
		colors: Float32Array;
		clip: Vector3[];
		range: Vector3[];
		onselect?: (voxel: number, rgb: [number, number, number], hex: string) => void;
	}

	const { matrices, colors, ndiv, clip, range, onselect }: Props = $props();

	// 相机角度双向同步：OrbitControls 变化写回 sceneProps；sceneProps 变化驱动相机
	let orbitControls: any = $state();
	const sameVec = (a: [number, number, number], b: [number, number, number]) =>
		a[0] === b[0] && a[1] === b[1] && a[2] === b[2];
	const syncCamera = () => {
		if (!orbitControls) return;
		const cam = orbitControls.object as PerspectiveCamera;
		const pos: [number, number, number] = [cam.position.x, cam.position.y, cam.position.z];
		const tgt: [number, number, number] = [
			orbitControls.target.x,
			orbitControls.target.y,
			orbitControls.target.z
		];
		// 内容相同时不赋值（$state 数组按引用比较），避免 damping 每帧更新造成环路
		if (!sameVec(sceneProps.cameraPos, pos)) sceneProps.cameraPos = pos;
		if (!sameVec(sceneProps.cameraTarget, tgt)) sceneProps.cameraTarget = tgt;
	};

	interactivity({
		filter(items) {
			for (const item of items) {
				if (item.instanceId !== undefined) {
					const i = item.instanceId * 16;
					const [x, y, z] = [matrices[i + 12], matrices[i + 13], matrices[i + 14]];
					// 体素立方体 [c-0.5, c+0.5] 与 clip 区间相交——与 shader 的 clipLow/High（±0.5）
					// 一致，否则边界体素（中心恰好等于 clip 边界，如蓝色 gz=-15 == clipB[0]）
					// 渲染可见却点不到。
					const inside = (c: number, lo: number, hi: number) => c - 0.5 < hi && c + 0.5 > lo;
					if (
						inside(x, clip[0].x, clip[1].x) &&
						inside(y, clip[0].y, clip[1].y) &&
						inside(z, clip[0].z, clip[1].z)
					) {
						return [item];
					}
				}
			}
			return [];
		}
	});

	let mesh: InstancedMesh | undefined = $state();
	let hoveredId = -1;

	const nvoxels = $derived(colors.length / 3);
	const scale = $derived(1 / (ndiv - 1));

	const gridLineMaterial = new LineBasicMaterial({
		color: 0x888888,
		clippingPlanes: [
			new Plane(new Vector3(1, 0, 0), 0.5)
			// new Plane(new Vector3(1, 0, 0), 0)
		]
	});

	const clipLow = $derived(clip[0].clone().addScalar(-0.5));
	const clipHigh = $derived(clip[1].clone().addScalar(-0.5));

	$effect(() => {
		const eps = 1e-3;
		gridLineMaterial.clippingPlanes = [
			new Plane(new Vector3(1, 0, 0), -1 * (clipLow.x * scale - 0.5) + eps),
			new Plane(new Vector3(0, 1, 0), -1 * (clipLow.y * scale) + eps),
			new Plane(new Vector3(0, 0, 1), -1 * (clipLow.z * scale) + eps),

			new Plane(new Vector3(-1, 0, 0), 1 * (clipHigh.x * scale - 0.5) + eps),
			new Plane(new Vector3(0, -1, 0), 1 * (clipHigh.y * scale) + eps),
			new Plane(new Vector3(0, 0, -1), 1 * (clipHigh.z * scale) + eps)
		];
		gridLineMaterial.needsUpdate = true;
		invalidate();
	});

	$effect(() => {
		if (!mesh) return;
		const mat = mesh.material as ShaderMaterial;
		if (!mat.uniforms) return;
		mat.uniforms.uClipLow.value.copy(clipLow);
		mat.uniforms.uClipHigh.value.copy(clipHigh);
		mat.uniformsNeedUpdate = true;
	});

	const writeMatrix = (i: number, s: number) => {
		if (!mesh) return;
		for (const j of [0, 5, 10]) {
			const idx = i * 16 + j;
			if (idx < matrices.length) {
				matrices[idx] = s;
			}
		}
	};

	// hover 动画：补间当前 scale 到目标（旧 hover 回 1，新 hover 放大到 1.3）
	const curScale = new Map<number, number>();
	const animTokens = new Map<number, number>();
	let animSeq = 0;
	const animateScale = (id: number, from: number, to: number, duration = 150) => {
		const token = ++animSeq;
		animTokens.set(id, token);
		const start = performance.now();
		const step = (now: number) => {
			if (animTokens.get(id) !== token) return; // 被新动画取代
			if (!mesh) {
				animTokens.delete(id);
				return;
			}
			const t = Math.min(1, (now - start) / duration);
			const eased = 1 - Math.pow(1 - t, 3); // easeOutCubic
			const s = from + (to - from) * eased;
			writeMatrix(id, s);
			curScale.set(id, s);
			mesh.instanceMatrix.needsUpdate = true;
			invalidate();
			if (t < 1) requestAnimationFrame(step);
			else animTokens.delete(id);
		};
		requestAnimationFrame(step);
	};

	const setHovered = (id: number) => {
		// hover 到 voxel 时光标变小手（pointermove 每帧触发，id 相同也保持）
		renderer.domElement.style.cursor = id >= 0 ? 'pointer' : '';
		if (id === hoveredId || !mesh) return;
		// 从当前实际 scale 补间（避免动画中途切走时跳变）
		if (hoveredId >= 0) animateScale(hoveredId, curScale.get(hoveredId) ?? 1.3, 1);
		if (id >= 0) animateScale(id, curScale.get(id) ?? 1, 1.3);
		hoveredId = id;
		invalidate();
	};
</script>

<T.PerspectiveCamera
	makeDefault
	fov={50}
	position={sceneProps.cameraPos}
	oncreate={(ref) => {
		ref.lookAt(...sceneProps.cameraTarget);
	}}
	zoom={sceneProps.zoom}
>
	<OrbitControls
		bind:ref={orbitControls}
		enableDamping={true}
		enableZoom={true}
		onchange={syncCamera}
	>
		<Gizmo x={{ label: 'L' }} y={{ label: 'a' }} z={{ label: 'b' }} />
	</OrbitControls>
</T.PerspectiveCamera>

<T.Group position={[-0.5, 0, 0]}>
	<!-- {#key nvoxels} -->
	<T.InstancedMesh
		{scale}
		bind:ref={mesh}
		args={[undefined, undefined, nvoxels]}
		onpointermove={(e: any) => setHovered(e.instanceId ?? -1)}
		onpointerleave={() => setHovered(-1)}
		onclick={(e: any) => {
			const id = e.instanceId ?? -1;
			if (id >= 0) {
				// colors 是 linear sRGB，需先经 sRGB 传递函数转成 8-bit 显示值
				const toByte = (c: number) => clamp(Math.round(linearToSrgb(c) * 255), 0, 255);
				const r = toByte(colors[id * 3]);
				const g = toByte(colors[id * 3 + 1]);
				const b = toByte(colors[id * 3 + 2]);
				const hex = `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, '0')}`;
				console.log('clicked voxel', id, hex);
				onselect?.(id, [r, g, b], hex);
			}
		}}
	>
		<T.InstancedBufferAttribute args={[matrices, 16]} attach="instanceMatrix" />
		<T.InstancedBufferAttribute args={[colors, 3]} attach="instanceColor" />
		<!-- <T.InstancedBufferAttribute args={[visible, 1]} attach="aVisible" /> -->
		<T.BoxGeometry args={[1, 1, 1]} />
		<T.ShaderMaterial
			uniforms={{
				uClipLow: { value: new Vector3() },
				uClipHigh: { value: new Vector3() }
			}}
			fragmentShader={`
					#include <common>
					#include <color_pars_fragment>

					varying float vVisible;

					void main() {
						if (vVisible < 0.5) discard;

						gl_FragColor = vColor;

						#include <tonemapping_fragment>
						#include <colorspace_fragment>
					}`}
			vertexShader={`
					#include <common>
					#include <color_pars_vertex>

					uniform vec3 uClipLow;
					uniform vec3 uClipHigh;

					varying float vVisible;

					void main() {
						vec3 coord = instanceMatrix[3].xyz;
						if (all(greaterThan(coord, uClipLow)) && all(lessThan(coord, uClipHigh))) {
							vVisible = 1.0;
						} else {
							vVisible = 0.0;
						}

						#include <color_vertex>
						#include <begin_vertex>
						#include <project_vertex>
					}`}
		/>
	</T.InstancedMesh>
	<!-- {/key} -->

	<T.ArrowHelper args={[new Vector3(1, 0, 0), new Vector3(0, 0, 0), 1.25, 0xff3653, 0.15]} />
	<T.ArrowHelper args={[new Vector3(0, 1, 0), new Vector3(0, 0, 0), 1, 0x8adb00, 0.15]} />
	<T.ArrowHelper args={[new Vector3(0, 0, 1), new Vector3(0, 0, 0), 1, 0x2c8fff, 0.15]} />
	<T.ArrowHelper args={[new Vector3(0, -1, 0), new Vector3(0, 0, 0), 1, 0x8adb00, 0]} />
	<T.ArrowHelper args={[new Vector3(0, 0, -1), new Vector3(0, 0, 0), 1, 0x2c8fff, 0]} />

	{@const ngrid = 2 * ndiv + 1}
	{#each [clip[0].x, clip[1].x] as v, i}
		{#if v != range[i].x}
			<T.GridHelper
				position={[(v - 0.5) * scale, 0, 0]}
				rotation={[0, 0, Math.PI / 2]}
				args={[1 * ngrid * scale, ngrid]}
				material={gridLineMaterial}
			/>
		{/if}
	{/each}

	{#each [clip[0].y, clip[1].y] as v, i}
		{#if v != range[i].y}
			<T.GridHelper
				position={[0.5 + 0.5 * scale, (v - 0.5) * scale, 0]}
				rotation={[0, Math.PI / 2, 0]}
				args={[1 * ngrid * scale, ngrid]}
				material={gridLineMaterial}
			/>
		{/if}
	{/each}

	{#each [clip[0].z, clip[1].z] as v, i}
		{#if v != range[i].z}
			<T.GridHelper
				position={[0.5 + 0.5 * scale, 0, (v - 0.5) * scale]}
				rotation={[Math.PI / 2, 0, 0]}
				args={[1 * ngrid * scale, ngrid]}
				material={gridLineMaterial}
			/>
		{/if}
	{/each}
</T.Group>
