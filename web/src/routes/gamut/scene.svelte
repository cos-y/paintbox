<script lang="ts">
	import { Gizmo, OrbitControls, interactivity } from '@threlte/extras';
	import { T, useThrelte } from '@threlte/core';
	import * as THREE from 'three';

	const { toneMapping, invalidate } = useThrelte();
	toneMapping.set(THREE.NoToneMapping);

	type MaybeNumber = number | undefined;

	interface Props {
		ndiv: number;
		matrices: Float32Array;
		colors: Float32Array;
		clipL: MaybeNumber[];
		clipA: MaybeNumber[];
		clipB: MaybeNumber[];
	}

	const { matrices, colors, ndiv, clipL, clipA, clipB }: Props = $props();

	interactivity({
		filter(items) {
			for (const item of items) {
				if (item.instanceId !== undefined) {
					const i = item.instanceId * 16;
					const [x, y, z] = [matrices[i + 12], matrices[i + 13], matrices[i + 14]];
					if (clipLow.x < x && x < clipHigh.x) {
						if (clipLow.y < y && y < clipHigh.y) {
							if (clipLow.z < z && z < clipHigh.z) {
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

	const clipLow = $derived.by(() => {
		const l = clipL[0] ?? -Infinity;
		const a = clipA[0] ?? -Infinity;
		const b = clipB[0] ?? -Infinity;
		return new THREE.Vector3(l, a, b).addScalar(-0.5);
	});

	const clipHigh = $derived.by(() => {
		const l = clipL[1] ?? Infinity;
		const a = clipA[1] ?? Infinity;
		const b = clipB[1] ?? Infinity;
		return new THREE.Vector3(l, a, b).addScalar(-0.5);
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
	position={[3, 2, -1]}
	oncreate={(ref) => {
		ref.lookAt(0, 0, 0);
	}}
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
					const r = (colors[id * 3] * 255) | 0;
					const g = (colors[id * 3 + 1] * 255) | 0;
					const b = (colors[id * 3 + 2] * 255) | 0;
					console.log(
						'clicked voxel',
						id,
						`#${((r << 16) | (g << 8) | b).toString(16).padStart(6, '0')}`
					);
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

	{@const ngrid = ndiv + 1}
	{#each clipL as v}
		{#if v !== undefined}
			<T.GridHelper
				position={[(v - 0.5) * scale, 0, 0]}
				rotation={[0, 0, Math.PI / 2]}
				args={[1 * ngrid * scale, ngrid]}
			/>
		{/if}
	{/each}

	{#each clipA as v}
		{#if v !== undefined}
			<T.GridHelper
				position={[0.5 + 0.5 * scale, (v - 0.5) * scale, 0]}
				rotation={[0, Math.PI / 2, 0]}
				args={[1 * ngrid * scale, ngrid]}
			/>
		{/if}
	{/each}

	{#each clipB as v}
		{#if v !== undefined}
			<T.GridHelper
				position={[0.5 + 0.5 * scale, 0, (v - 0.5) * scale]}
				rotation={[Math.PI / 2, 0, 0]}
				args={[1 * ngrid * scale, ngrid]}
			/>
		{/if}
	{/each}
</T.Group>
