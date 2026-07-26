<script lang="ts">
	import { Gizmo, OrbitControls } from '@threlte/extras';
	import { T, useThrelte } from '@threlte/core';
	import { get_hull, get_srgb_mesh } from '../../wasm-pkg/paintbox_wasm';
	import * as THREE from 'three';
	import { listPaints, rgbToHex } from '$lib/paints';

	const { toneMapping } = useThrelte();
	toneMapping.set(THREE.NoToneMapping);

	// let r = 0xff0000;
	// let g = 0x00ff00;
	// let b = 0x0000ff;
	// let c = 0x00ffff;
	// let m = 0xff00ff;
	// let y = 0xffff00;
	// let k = 0x000000;
	// let w = 0xffffff;
	// let gr = 0xaaaaaa;

	let r = 0xed1c24;
	let g = 0x00a650;
	let b = 0x005aaa;
	let c = 0x00aeef;
	let m = 0xec008c;
	let y = 0xfff200;
	let k = 0x231f20;
	let w = 0xffffff;

	// let li = [r, g, b, c, m, y, k, w];
	let li = listPaints().map((x) => x.rgb);
	console.log(li.map((x) => rgbToHex(x)));

	const li1 = [li[1], li[0], li[4], li[2]];
	li.splice(5);
	// li = [r, g, b, c];
	// li.splice(50);
	// const [_1] = li.splice(3, 1);
	// const [_2] = li.splice(7, 1);
	// // const [_3] = li.splice(7, 1);

	// const hull = get_hull(new Uint32Array(li1));
	// hull.add(li[7]);
	const hull = get_hull(new Uint32Array(li));
	hull.add(r);
	hull.add(g);
	hull.add(b);
	hull.add(c);
	hull.add(m);
	hull.add(y);
	hull.add(k);
	hull.add(w);

	let isModified = $state(false);

	(window as any).add = (v: number) => {
		isModified = true;
		hull.add(v);
	};

	$effect(() => {
		if (isModified) {
			isModified = false;
			mesh = hull.mesh();
		}
	});

	let mesh = $state(hull.mesh());
	let positions = $derived(mesh.positions());
	let colors = $derived(mesh.colors());
	let indices = $derived(mesh.indices());

	const srgbMesh = get_srgb_mesh(8);
	let srgbPositions = $derived(srgbMesh.positions());
	let srgbColors = $derived(srgbMesh.colors());
	let srgbIndices = $derived(srgbMesh.indices());
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
		<Gizmo />
	</OrbitControls>
</T.PerspectiveCamera>
<!-- 
<T.Mesh scale={0.01} position={[-0.5, 0, 0]}>
	<T.BufferGeometry attach="geometry">
		<T.BufferAttribute args={[positions, 3]} attach="attributes.position" />
		<T.BufferAttribute args={[colors, 3]} attach="attributes.color" />
		<T.BufferAttribute args={[indices, 1]} attach="index" />
	</T.BufferGeometry>
	<T.MeshBasicMaterial side={1} vertexColors={true} />
</T.Mesh> -->

<T.Mesh scale={0.01} position={[-0.5, 0, 0]}>
	<T.BufferGeometry attach="geometry">
		<T.BufferAttribute args={[positions, 3]} attach="attributes.position" />
		<T.BufferAttribute args={[colors, 3]} attach="attributes.color" />
		<T.BufferAttribute args={[indices, 1]} attach="index" />
	</T.BufferGeometry>
	<T.MeshBasicMaterial side={2} vertexColors={true} wireframe={true} />
</T.Mesh>

<!-- <T.Mesh scale={0.01} position={[-0.5, 0, 0]}>
	<T.BufferGeometry attach="geometry">
		<T.BufferAttribute args={[srgbPositions, 3]} attach="attributes.position" />
		<T.BufferAttribute args={[srgbColors, 3]} attach="attributes.color" />
		<T.BufferAttribute args={[srgbIndices, 1]} attach="index" />
	</T.BufferGeometry>
	<T.MeshBasicMaterial side={2} vertexColors={true} wireframe={true} />
</T.Mesh> -->

<T.GridHelper args={[2, 20]} />
