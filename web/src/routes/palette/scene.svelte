<script lang="ts">
	import { Gizmo, OrbitControls } from '@threlte/extras';
	import { T, useThrelte } from '@threlte/core';
	import { hull } from '../../wasm-pkg/paintbox_wasm';
	import * as THREE from 'three';
	import { color } from 'three/tsl';

	const { renderer, toneMapping } = useThrelte();

	// THREE.ColorManagement.enabled = false;
	// renderer.outputColorSpace = THREE.LinearSRGBColorSpace;
	toneMapping.set(THREE.NoToneMapping);

	// const { renderer } = useThrelte();
	// renderer.outputColorSpace = THREE.LinearSRGBColorSpace;

	let r = 0xff0000;
	let g = 0x00ff00;
	let b = 0x0000ff;
	let c = 0x00ffff;
	let m = 0xff00ff;
	let y = 0xffff00;
	let k = 0x000000;
	let w = 0xffffff;
	let gr = 0xaaaaaa;

	// let r = 0xed1c24;
	// let g = 0x00a650;
	// let b = 0x005aaa;
	// let c = 0x00aeef;
	// let m = 0xec008c;
	// let y = 0xfff200;
	// let k = 0x231f20;
	// let w = 0xffffff;

	// let x = 0xaaac3c;

	// const sim = demo(new Uint32Array([r, g, b, k]));
	// const sim = demo(new Uint32Array([r, g, b, c, m, y, k, w]), 100);
	// const sim = demo(new Uint32Array([r, y, c, b, k]), 100);
	// const sim = demo(new Uint32Array([0xff00ff, 0x0000ff, 0x0000ff]), 1);
	// const sim = hull(new Uint32Array([c, r, k, w]));
	const sim = hull(new Uint32Array([r, g, b, c, m, y, k, w]));

	// sim.add(0xaaaaaa);

	let positions = $state(sim.points());
	let colors = $state(sim.colors());
	let indices = $state(sim.indices());

	// (window as any).add = (rgb: number) => {
	// 	sim.add(rgb);
	// 	positions = sim.points();
	// 	colors = sim.colors();
	// 	indices = sim.indices();
	// };
	// $effect(() => {
	// 	let maxx = 0;
	// 	for (let i = 0; i < indices.length; i += 3) {
	// 		let a = indices[i];
	// 		let b = indices[i + 1];
	// 		let c = indices[i + 2];
	// 		console.log(
	// 			[positions[a * 3], positions[a * 3 + 1], positions[a * 3 + 2]],
	// 			[positions[b * 3], positions[b * 3 + 1], positions[b * 3 + 2]],
	// 			[positions[c * 3], positions[c * 3 + 1], positions[c * 3 + 2]]
	// 		);
	// 	}
	// 	// console.log(indices);
	// });
</script>

<T.PerspectiveCamera
	makeDefault
	fov={50}
	position={[3, 2, 2]}
	oncreate={(ref) => {
		ref.lookAt(0, 0, 0);
	}}
>
	<OrbitControls enableDamping={true} enableZoom={true}>
		<Gizmo />
	</OrbitControls>
</T.PerspectiveCamera>

<T.Mesh scale={0.01} position={[-0.5, 0, 0]}>
	<T.BufferGeometry attach="geometry">
		<T.BufferAttribute args={[positions, 3]} attach="attributes.position" />
		<T.BufferAttribute args={[colors, 3]} attach="attributes.color" />
		<T.BufferAttribute args={[indices, 1]} attach="index" />
	</T.BufferGeometry>
	<T.MeshBasicMaterial size={0.05} vertexColors={true} side={2} wireframe={true} />
</T.Mesh>

<!-- 
<T.Points>
	<T.BufferGeometry attach="geometry">
		<T.BufferAttribute args={[positions, 3]} attach="attributes.position" />
		<T.BufferAttribute args={[colors, 3]} attach="attributes.color" />
	</T.BufferGeometry>
	<T.PointsMaterial size={0.05} vertexColors={true} />
</T.Points> -->

<T.GridHelper args={[2, 20]} />
