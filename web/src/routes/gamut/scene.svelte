<script lang="ts">
	import { Gizmo, OrbitControls, interactivity } from '@threlte/extras';
	import { T, useThrelte } from '@threlte/core';
	import * as THREE from 'three';

	const { toneMapping, invalidate } = useThrelte();
	toneMapping.set(THREE.NoToneMapping);

	interface Props {
		ndiv: number;
		matrices: Float32Array;
		colors: Float32Array;
	}

	const { matrices, colors, ndiv }: Props = $props();

	interactivity({
		filter(items) {
			return items.slice(0, 1);
		}
	});

	let mesh: THREE.InstancedMesh | undefined = $state();
	let hoveredId = -1;
	const dummy = new THREE.Object3D();

	const nvoxels = $derived(colors.length / 3);

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
			scale={1 / ndiv}
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
			<T.BoxGeometry args={[1, 1, 1]} />
			<T.MeshBasicMaterial />
		</T.InstancedMesh>
	{/key}

	<!-- <T.GridHelper position={[0.5, 0, 0]} args={[(1 * (ndiv + 2)) / ndiv, ndiv + 2]} /> -->
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
</T.Group>
