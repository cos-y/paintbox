<script lang="ts">
	import { Gizmo, OrbitControls, interactivity } from '@threlte/extras';
	import { T, useThrelte } from '@threlte/core';
	import { get_hull } from '../../wasm-pkg/paintbox_wasm';
	import * as THREE from 'three';
	import { listPaints, rgbToHex } from '$lib/paints';

	const { toneMapping, invalidate } = useThrelte();
	toneMapping.set(THREE.NoToneMapping);

	// 启用指针事件（射线拾取），只取最近的一个命中
	interactivity({
		filter(items) {
			return items.slice(0, 1);
		}
	});

	let r = 0xff0000;
	let g = 0x00ff00;
	let b = 0x0000ff;
	let c = 0x00ffff;
	let m = 0xff00ff;
	let y = 0xffff00;
	let k = 0x000000;
	let w = 0xffffff;

	let li = listPaints().map((x) => x.rgb);
	li.splice(100);
	console.log(li);

	console.time();
	const ndiv = 16;
	const hull = get_hull(new Uint32Array(li), 100 / ndiv);
	console.timeEnd();

	console.time();
	hull.add(r);
	hull.add(g);
	hull.add(b);
	// hull.add(c);
	// hull.add(m);
	// hull.add(y);
	// hull.add(k);
	console.timeEnd();

	let mesh: THREE.InstancedMesh | undefined = $state();

	// 一次取数据、算好实例数（count 是构造时固定的）
	const indices = hull.indices();
	const colors = hull.colors();
	const nvoxels = colors.length;
	console.log('nvoxels =', nvoxels);

	let hoveredId = -1;
	const dummy = new THREE.Object3D();

	// 写入第 i 个实例的变换（s = 缩放倍率）
	const writeMatrix = (i: number, s: number) => {
		if (!mesh) return;
		dummy.position.set(indices[i * 3 + 0], indices[i * 3 + 1], indices[i * 3 + 2]);
		dummy.scale.setScalar(s);
		dummy.updateMatrix();
		mesh.setMatrixAt(i, dummy.matrix);
	};

	const fill = () => {
		if (!mesh) return;
		for (let i = 0; i < nvoxels; ++i) {
			writeMatrix(i, 1);
			mesh.setColorAt(i, new THREE.Color(colors[i]));
		}
		mesh.instanceMatrix.needsUpdate = true;
		if (mesh.instanceColor) mesh.instanceColor.needsUpdate = true;
		invalidate();
	};

	// mesh 挂载后填充实例数据
	$effect(() => {
		fill();
	});

	// 悬停高亮：只重写变化的两个实例，不遍历全部
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
		<Gizmo />
	</OrbitControls>
</T.PerspectiveCamera>

<T.InstancedMesh
	scale={1 / ndiv}
	position={[-0.5, 0, 0]}
	bind:ref={mesh}
	args={[undefined, undefined, nvoxels]}
	onpointermove={(e: any) => setHovered(e.instanceId ?? -1)}
	onpointerleave={() => setHovered(-1)}
	onclick={(e: any) => {
		const id = e.instanceId ?? -1;
		if (id >= 0) console.log('clicked voxel', id, '#' + colors[id].toString(16).padStart(6, '0'));
	}}
>
	<T.BoxGeometry args={[1, 1, 1]} />
	<T.MeshBasicMaterial />
</T.InstancedMesh>

<T.GridHelper args={[2, 20]} />
