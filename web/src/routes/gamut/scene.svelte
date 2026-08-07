<script lang="ts">
	import { Gizmo, OrbitControls, interactivity } from '@threlte/extras';
	import { T, useThrelte } from '@threlte/core';
	import * as THREE from 'three';
	import { clamp, linearToSrgb } from '$lib/utils.svelte';

	const { toneMapping, invalidate, renderer } = useThrelte();
	toneMapping.set(THREE.NoToneMapping);
	renderer.localClippingEnabled = true;

	interface Props {
		ndiv: number;
		matrices: Float32Array;
		colors: Float32Array;
		clip: THREE.Vector3[];
		range: THREE.Vector3[];
		defaultZoom?: number;
		onselect?: (rgb: [number, number, number], hex: string) => void;
	}

	const { matrices, colors, ndiv, clip, range, defaultZoom = 1, onselect }: Props = $props();

	const zoom = $state(defaultZoom);

	interactivity({
		filter(items) {
			for (const item of items) {
				if (item.instanceId !== undefined) {
					const i = item.instanceId * 16;
					const [x, y, z] = [matrices[i + 12], matrices[i + 13], matrices[i + 14]];
					if (clip[0].x < x && x < clip[1].x) {
						if (clip[0].y < y && y < clip[1].y) {
							if (clip[0].z < z && z < clip[1].z) {
								return [item];
							}
						}
					}
				}
			}
			return [];
		}
	});

	let mesh: THREE.InstancedMesh | undefined = $state();
	let hoveredId = -1;

	const nvoxels = $derived(colors.length / 3);
	const scale = $derived(1 / (ndiv - 1));

	const gridLineMaterial = new THREE.LineBasicMaterial({
		color: 0x888888,
		clippingPlanes: [
			new THREE.Plane(new THREE.Vector3(1, 0, 0), 0.5)
			// new THREE.Plane(new THREE.Vector3(1, 0, 0), 0)
		]
	});

	const clipLow = $derived(clip[0].clone().addScalar(-0.5));
	const clipHigh = $derived(clip[1].clone().addScalar(-0.5));

	$effect(() => {
		const eps = 1e-3;
		gridLineMaterial.clippingPlanes = [
			new THREE.Plane(new THREE.Vector3(1, 0, 0), -1 * (clipLow.x * scale - 0.5) + eps),
			new THREE.Plane(new THREE.Vector3(0, 1, 0), -1 * (clipLow.y * scale) + eps),
			new THREE.Plane(new THREE.Vector3(0, 0, 1), -1 * (clipLow.z * scale) + eps),

			new THREE.Plane(new THREE.Vector3(-1, 0, 0), 1 * (clipHigh.x * scale - 0.5) + eps),
			new THREE.Plane(new THREE.Vector3(0, -1, 0), 1 * (clipHigh.y * scale) + eps),
			new THREE.Plane(new THREE.Vector3(0, 0, -1), 1 * (clipHigh.z * scale) + eps)
		];
		gridLineMaterial.needsUpdate = true;
		invalidate();
	});

	$effect(() => {
		if (!mesh) return;
		const mat = mesh.material as THREE.ShaderMaterial;
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

	const setHovered = (id: number) => {
		if (id === hoveredId || !mesh) return;
		if (hoveredId >= 0) writeMatrix(hoveredId, 1);
		if (id >= 0) writeMatrix(id, 1.3);
		hoveredId = id;
		mesh.instanceMatrix.needsUpdate = true;
		invalidate();
	};
</script>

<T.PerspectiveCamera
	makeDefault
	fov={50}
	position={[3, 0.5, 1]}
	oncreate={(ref) => {
		ref.lookAt(0, 0, 0);
	}}
	{zoom}
>
	<OrbitControls enableDamping={true} enableZoom={true}>
		<Gizmo x={{ label: 'L' }} y={{ label: 'a' }} z={{ label: 'b' }} />
	</OrbitControls>
</T.PerspectiveCamera>

<T.Group position={[-0.5, 0, 0]}>
	{#key nvoxels}
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
					onselect?.([r, g, b], hex);
				}
			}}
		>
			<T.InstancedBufferAttribute args={[matrices, 16]} attach="instanceMatrix" />
			<T.InstancedBufferAttribute args={[colors, 3]} attach="instanceColor" />
			<!-- <T.InstancedBufferAttribute args={[visible, 1]} attach="aVisible" /> -->
			<T.BoxGeometry args={[1, 1, 1]} />
			<T.ShaderMaterial
				uniforms={{
					uClipLow: { value: new THREE.Vector3() },
					uClipHigh: { value: new THREE.Vector3() }
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
	{/key}

	<T.ArrowHelper
		args={[new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 0, 0), 1.25, 0xff3653, 0.15]}
	/>
	<T.ArrowHelper
		args={[new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, 0, 0), 1, 0x8adb00, 0.15]}
	/>
	<T.ArrowHelper
		args={[new THREE.Vector3(0, 0, 1), new THREE.Vector3(0, 0, 0), 1, 0x2c8fff, 0.15]}
	/>
	<T.ArrowHelper args={[new THREE.Vector3(0, -1, 0), new THREE.Vector3(0, 0, 0), 1, 0x8adb00, 0]} />
	<T.ArrowHelper args={[new THREE.Vector3(0, 0, -1), new THREE.Vector3(0, 0, 0), 1, 0x2c8fff, 0]} />

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
