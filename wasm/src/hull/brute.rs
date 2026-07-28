//! 暴力但可靠的可达色域边界网格。
//!
//! 思路：把"所有可能的混合结果"当作 Lab 空间里的一个实心体，
//! 用体素占据栅格逼近它，再从二值栅格提取边界曲面。
//! 不做任何解析假设（凸性/流形选择），可靠性只取决于采样密度和体素分辨率。
//!
//! 流程：
//! 1. 采样：顶点 + 所有两色曲线 + 所有三色 patch（间距 < 半体素，保证外壳致密），
//!    外加随机子集 Dirichlet 混合，覆盖内部和 4+ 色混合面；
//! 2. 体素化 + 形态学闭运算（封采样小孔）；
//! 3. 从栅格外边界泛洪标记"外部"，取反得到实心体（内部空腔自动填实）；
//! 4. naive surface nets 提取边界（对二值场水密），拉普拉斯平滑去块状感。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use fixedbitset::FixedBitSet;
use glam::Vec3;
use lab::Lab;
use mixbox::{float_rgb_to_latent, latent_to_float_rgb};

use crate::{BoxError, Latent, Rgb, hull::Hull, log, mesh::Mesh};

fn latent_to_lab_vec(latent: &Latent) -> Vec3 {
    let rgb = latent_to_float_rgb(latent);
    let lab = Lab::from_rgb_normalized(&rgb);
    Vec3::new(lab.l, lab.a, lab.b)
}

// ── 参数 ─────────────────────────────────────────────────────────────────────

/// 栅格最长边的目标格数
// const TARGET_DIM: f32 = 96.0;
const TARGET_DIM: f32 = 16.0;
const VOXEL_MIN: f32 = 4.0;
const VOXEL_MAX: f32 = 8.0;
/// 随机混合样本数
const RANDOM_SAMPLES: usize = if cfg!(test) { 60_000 } else { 300_000 };
/// 随机混合的最大组分数
const RANDOM_MIX_MAX: usize = 6;
/// 拉普拉斯平滑轮数
const SMOOTH_ITERS: usize = 5;

// ── 随机数 ────────────────────────────────────────────────────────────────────

struct Rng {
    buf: Vec<u32>,
    at: usize,
}

impl Rng {
    fn new() -> Self {
        Rng { buf: vec![], at: 0 }
    }

    fn next_u32(&mut self) -> u32 {
        if self.at >= self.buf.len() {
            self.buf.resize(4096, 0);
            let bytes = unsafe {
                std::slice::from_raw_parts_mut(self.buf.as_mut_ptr() as *mut u8, 4096 * 4)
            };
            getrandom::fill(bytes).unwrap();
            self.at = 0;
        }
        let v = self.buf[self.at];
        self.at += 1;
        v
    }

    /// (0, 1]，避开 0 使 -ln(u) 有限
    fn next_unit(&mut self) -> f32 {
        (self.next_u32() as f64 + 1.0) as f32 / (u32::MAX as f64 + 1.0) as f32
    }

    fn next_below(&mut self, n: usize) -> usize {
        (self.next_u32() as usize) % n
    }
}

// ── 体素栅格 ──────────────────────────────────────────────────────────────────

struct VoxelGrid {
    origin: Vec3,
    voxel: f32,
    dims: [usize; 3],
    bits: FixedBitSet,
}

impl VoxelGrid {
    fn new(min: Vec3, max: Vec3) -> Self {
        let extent = (max - min).max_element();
        let voxel = (extent / TARGET_DIM).clamp(VOXEL_MIN, VOXEL_MAX);
        // 两侧各留 2 格 padding，保证边界格永远是空的（泛洪起点、surface nets 不越界）
        let pad = 2.0 * voxel;
        let origin = min - pad;
        let size = max - min + 2.0 * pad;
        let dims = [
            (size.x / voxel).ceil() as usize + 1,
            (size.y / voxel).ceil() as usize + 1,
            (size.z / voxel).ceil() as usize + 1,
        ];
        let bits = FixedBitSet::with_capacity(dims[0] * dims[1] * dims[2]);
        VoxelGrid {
            origin,
            voxel,
            dims,
            bits,
        }
    }

    #[inline]
    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        (z * self.dims[1] + y) * self.dims[0] + x
    }

    #[inline]
    fn get(&self, x: usize, y: usize, z: usize) -> bool {
        self.bits.contains(self.index(x, y, z))
    }

    fn stamp(&mut self, p: Vec3) {
        let q = (p - self.origin) / self.voxel;
        let (x, y, z) = (q.x as usize, q.y as usize, q.z as usize);
        if x < self.dims[0] && y < self.dims[1] && z < self.dims[2] {
            let i = self.index(x, y, z);
            self.bits.insert(i);
        }
    }

    /// 栅格点坐标 → Lab 空间坐标
    fn point(&self, x: usize, y: usize, z: usize) -> Vec3 {
        self.origin + Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5) * self.voxel
    }

    /// 形态学闭运算：6 邻域 dilate 一轮 + erode 一轮，封住单体素小孔
    fn close(&mut self) {
        let dilated = self.dilate6(&self.bits);
        self.bits = self.erode6(&dilated);
    }

    fn dilate6(&self, src: &FixedBitSet) -> FixedBitSet {
        let mut out = src.clone();
        let [nx, ny, nz] = self.dims;
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    if src.contains(self.index(x, y, z)) {
                        if x > 0 {
                            out.insert(self.index(x - 1, y, z));
                        }
                        if x + 1 < nx {
                            out.insert(self.index(x + 1, y, z));
                        }
                        if y > 0 {
                            out.insert(self.index(x, y - 1, z));
                        }
                        if y + 1 < ny {
                            out.insert(self.index(x, y + 1, z));
                        }
                        if z > 0 {
                            out.insert(self.index(x, y, z - 1));
                        }
                        if z + 1 < nz {
                            out.insert(self.index(x, y, z + 1));
                        }
                    }
                }
            }
        }
        out
    }

    fn erode6(&self, src: &FixedBitSet) -> FixedBitSet {
        let mut out = FixedBitSet::with_capacity(src.len());
        let [nx, ny, nz] = self.dims;
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let keep = src.contains(self.index(x, y, z))
                        && (x == 0 || src.contains(self.index(x - 1, y, z)))
                        && (x + 1 >= nx || src.contains(self.index(x + 1, y, z)))
                        && (y == 0 || src.contains(self.index(x, y - 1, z)))
                        && (y + 1 >= ny || src.contains(self.index(x, y + 1, z)))
                        && (z == 0 || src.contains(self.index(x, y, z - 1)))
                        && (z + 1 >= nz || src.contains(self.index(x, y, z + 1)));
                    // 边界格 padding 永远是空的，erode 会把它们清掉，无所谓
                    if keep {
                        out.insert(self.index(x, y, z));
                    }
                }
            }
        }
        out
    }

    /// 修成 well-composed：消除非流形的对角接触配置。
    /// C1：某 2×2 面内两实心体素仅对角接触；C2：2×2×2 内仅体对角接触。
    /// 只往里加体素，单调收敛。well-composed 体的 surface nets 输出保证水密流形。
    fn well_compose(&mut self) {
        let [nx, ny, nz] = self.dims;
        loop {
            let mut changed = false;

            // C1：三个方向的 2×2 面
            for &(a, b) in &[(0usize, 1usize), (1, 2), (0, 2)] {
                let mut p = [0usize; 3];
                let c = 3 - a - b; // 剩下的轴
                for i in 0..self.dims[c] {
                    for j in 0..self.dims[a] - 1 {
                        for k in 0..self.dims[b] - 1 {
                            p[c] = i;
                            p[a] = j;
                            p[b] = k;
                            let mut q = p;
                            q[a] += 1;
                            let mut r = p;
                            r[b] += 1;
                            let mut s = p;
                            s[a] += 1;
                            s[b] += 1;
                            let sp = self.get(p[0], p[1], p[2]);
                            let sq = self.get(q[0], q[1], q[2]);
                            let sr = self.get(r[0], r[1], r[2]);
                            let ss = self.get(s[0], s[1], s[2]);
                            if sp && ss && !sq && !sr {
                                let i = self.index(q[0], q[1], q[2]);
                                self.bits.insert(i);
                                changed = true;
                            } else if sq && sr && !sp && !ss {
                                let i = self.index(p[0], p[1], p[2]);
                                self.bits.insert(i);
                                changed = true;
                            }
                        }
                    }
                }
            }

            // C2：2×2×2 体对角（仅两实心且互为体对角 / 仅两空且互为体对角）
            for z in 0..nz - 1 {
                for y in 0..ny - 1 {
                    for x in 0..nx - 1 {
                        let mut solid = [false; 8];
                        let mut count = 0;
                        for (bi, (dx, dy, dz)) in [
                            (0, 0, 0),
                            (1, 0, 0),
                            (0, 1, 0),
                            (1, 1, 0),
                            (0, 0, 1),
                            (1, 0, 1),
                            (0, 1, 1),
                            (1, 1, 1),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            solid[bi] = self.get(x + dx, y + dy, z + dz);
                            count += solid[bi] as usize;
                        }
                        // 体对角对：(0,7) (1,6) (2,5) (3,4)
                        for (u, v) in [(0usize, 7usize), (1, 6), (2, 5), (3, 4)] {
                            if count == 2 && solid[u] && solid[v] {
                                // 填一个相邻体素打通连接
                                let i = self.index(x + 1, y, z);
                                self.bits.insert(i);
                                changed = true;
                            } else if count == 6 && !solid[u] && !solid[v] {
                                let (dx, dy, dz) = [
                                    (0, 0, 0),
                                    (1, 0, 0),
                                    (0, 1, 0),
                                    (1, 1, 0),
                                    (0, 0, 1),
                                    (1, 0, 1),
                                    (0, 1, 1),
                                    (1, 1, 1),
                                ][u];
                                let i = self.index(x + dx, y + dy, z + dz);
                                self.bits.insert(i);
                                changed = true;
                            }
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }
    }

    /// 从栅格外边界泛洪空体素标记"外部"，然后取反：实心体 = 非外部。
    /// 被外壳封住的内部空腔自动算内部。
    fn solidify(&mut self) {
        let [nx, ny, nz] = self.dims;
        let mut outside = FixedBitSet::with_capacity(self.bits.len());
        let mut queue: Vec<(usize, usize, usize)> = vec![];

        let mut try_push =
            |x: usize, y: usize, z: usize, outside: &mut FixedBitSet, queue: &mut Vec<_>| {
                let i = (z * ny + y) * nx + x;
                if !self.bits.contains(i) && !outside.contains(i) {
                    outside.insert(i);
                    queue.push((x, y, z));
                }
            };

        // 六个面作为泛洪种子
        for z in 0..nz {
            for y in 0..ny {
                try_push(0, y, z, &mut outside, &mut queue);
                try_push(nx - 1, y, z, &mut outside, &mut queue);
            }
        }
        for z in 0..nz {
            for x in 0..nx {
                try_push(x, 0, z, &mut outside, &mut queue);
                try_push(x, ny - 1, z, &mut outside, &mut queue);
            }
        }
        for y in 0..ny {
            for x in 0..nx {
                try_push(x, y, 0, &mut outside, &mut queue);
                try_push(x, y, nz - 1, &mut outside, &mut queue);
            }
        }

        while let Some((x, y, z)) = queue.pop() {
            let mut visit = |x: usize, y: usize, z: usize| {
                let i = (z * ny + y) * nx + x;
                if !self.bits.contains(i) && !outside.contains(i) {
                    outside.insert(i);
                    queue.push((x, y, z));
                }
            };
            if x > 0 {
                visit(x - 1, y, z);
            }
            if x + 1 < nx {
                visit(x + 1, y, z);
            }
            if y > 0 {
                visit(x, y - 1, z);
            }
            if y + 1 < ny {
                visit(x, y + 1, z);
            }
            if z > 0 {
                visit(x, y, z - 1);
            }
            if z + 1 < nz {
                visit(x, y, z + 1);
            }
        }

        // 实心 = 非外部
        let mut solid = FixedBitSet::with_capacity(self.bits.len());
        for i in 0..self.bits.len() {
            if !outside.contains(i) {
                solid.insert(i);
            }
        }
        self.bits = solid;
    }
}

// ── 采样 ─────────────────────────────────────────────────────────────────────

/// 两色曲线上按 Lab 弧长自适应采样
fn sample_pair(a: &Latent, b: &Latent, spacing: f32, out: &mut Vec<Vec3>) {
    let la = latent_to_lab_vec(a);
    let lb = latent_to_lab_vec(b);
    // 弦长下界估计弧长，×2 冗余
    let n = ((la.distance(lb) / spacing).ceil() as usize).max(2);
    for k in 0..=n {
        let t = k as f32 / n as f32;
        let l: Latent = std::array::from_fn(|d| (1.0 - t) * a[d] + t * b[d]);
        out.push(latent_to_lab_vec(&l));
    }
}

/// 三色 patch 上按重心坐标网格采样
fn sample_triple(a: &Latent, b: &Latent, c: &Latent, spacing: f32, out: &mut Vec<Vec3>) {
    let la = latent_to_lab_vec(a);
    let lb = latent_to_lab_vec(b);
    let lc = latent_to_lab_vec(c);
    let ext = la.distance(lb).max(lb.distance(lc)).max(la.distance(lc));
    let n = ((ext / spacing).ceil() as usize).max(2);
    for i in 0..=n {
        for j in 0..=(n - i) {
            let u = i as f32 / n as f32;
            let v = j as f32 / n as f32;
            let w = 1.0 - u - v;
            let l: Latent = std::array::from_fn(|d| u * a[d] + v * b[d] + w * c[d]);
            out.push(latent_to_lab_vec(&l));
        }
    }
}

/// 随机子集 Dirichlet 混合：覆盖内部和 4+ 色混合面
fn sample_random(latents: &[Latent], count: usize, rng: &mut Rng, out: &mut Vec<Vec3>) {
    let n = latents.len();
    let mut idx = [0usize; RANDOM_MIX_MAX];
    let mut w = [0f32; RANDOM_MIX_MAX];
    for _ in 0..count {
        let k = 2 + rng.next_below(RANDOM_MIX_MAX.min(n) - 1);
        let mut sum = 0f32;
        for s in 0..k {
            idx[s] = rng.next_below(n);
            // Dirichlet(1..1) = 归一化指数分布
            let e = -rng.next_unit().ln();
            w[s] = e;
            sum += e;
        }
        let l: Latent =
            std::array::from_fn(|d| (0..k).map(|s| latents[idx[s]][d] * w[s] / sum).sum());
        out.push(latent_to_lab_vec(&l));
    }
}

// ── 边界面提取 ───────────────────────────────────────────────────────────────

/// 提取实心体的体素边界面（blocky quad mesh）。
/// well-composed 实心体的边界面数学上保证是流形且水密：
/// 每个实心体素朝空邻居的一面生成一个 quad，共享的顶点按整数角坐标去重。
fn boundary_quads(grid: &VoxelGrid) -> (Vec<Vec3>, Vec<usize>) {
    let [nx, ny, nz] = grid.dims;

    let mut corner_id: HashMap<(usize, usize, usize), usize> = HashMap::new();
    let mut positions: Vec<Vec3> = vec![];
    let mut indices: Vec<usize> = vec![];

    // 角坐标 (cx,cy,cz) → Lab 空间坐标（体素中心是 (x+0.5)*voxel，故角是整数倍）
    let corner_pos = |cx: usize, cy: usize, cz: usize| -> Vec3 {
        grid.origin + Vec3::new(cx as f32, cy as f32, cz as f32) * grid.voxel
    };

    // 六个面朝外 CCW 的角偏移
    const FACES: [((isize, isize, isize), [(usize, usize, usize); 4]); 6] = [
        // +X
        ((1, 0, 0), [(1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)]),
        // -X
        ((-1, 0, 0), [(0, 0, 0), (0, 0, 1), (0, 1, 1), (0, 1, 0)]),
        // +Y
        ((0, 1, 0), [(0, 1, 0), (0, 1, 1), (1, 1, 1), (1, 1, 0)]),
        // -Y
        ((0, -1, 0), [(0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)]),
        // +Z
        ((0, 0, 1), [(0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)]),
        // -Z
        ((0, 0, -1), [(0, 0, 0), (0, 1, 0), (1, 1, 0), (1, 0, 0)]),
    ];

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                if !grid.get(x, y, z) {
                    continue;
                }
                for ((dx, dy, dz), corners) in FACES {
                    let (nxp, nyp, nzp) = (x as isize + dx, y as isize + dy, z as isize + dz);
                    // 邻居实心则此面在内部，跳过；越界当作空（外部）
                    let neighbor_solid = nxp >= 0
                        && nyp >= 0
                        && nzp >= 0
                        && (nxp as usize) < nx
                        && (nyp as usize) < ny
                        && (nzp as usize) < nz
                        && grid.get(nxp as usize, nyp as usize, nzp as usize);
                    if neighbor_solid {
                        continue;
                    }

                    let mut quad = [0usize; 4];
                    for (i, (ox, oy, oz)) in corners.into_iter().enumerate() {
                        let key = (x + ox, y + oy, z + oz);
                        quad[i] = *corner_id.entry(key).or_insert_with(|| {
                            positions.push(corner_pos(key.0, key.1, key.2));
                            positions.len() - 1
                        });
                    }
                    indices.extend_from_slice(&[quad[0], quad[1], quad[2]]);
                    indices.extend_from_slice(&[quad[0], quad[2], quad[3]]);
                }
            }
        }
    }

    (positions, indices)
}

/// 拉普拉斯平滑（λ=0.5），保持拓扑不变
fn smooth(positions: &mut Vec<Vec3>, indices: &[usize], iters: usize) {
    let n = positions.len();
    let mut neighbors: Vec<Vec<usize>> = vec![vec![]; n];
    for tri in indices.chunks(3) {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }
    for _ in 0..iters {
        let snapshot = positions.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let avg =
                neighbors[i].iter().map(|&j| snapshot[j]).sum::<Vec3>() / neighbors[i].len() as f32;
            positions[i] = (snapshot[i] + avg) * 0.5;
        }
    }
}

// ── Hull 实现 ─────────────────────────────────────────────────────────────────

pub struct BruteHull {
    colors: Vec<Rgb>,
    mesh: Arc<Mutex<Mesh>>,
}

impl BruteHull {
    fn rebuild(&mut self) {
        let n = self.colors.len();
        let mut mesh = self.mesh.lock().unwrap();
        mesh.clear();
        if n == 0 {
            return;
        }

        let latents: Vec<Latent> = self.colors.iter().map(|c| float_rgb_to_latent(c)).collect();

        // 1. 顶点样本，先确定 bbox
        let verts: Vec<Vec3> = latents.iter().map(|l| latent_to_lab_vec(l)).collect();
        let mut min = verts[0];
        let mut max = verts[0];
        for v in &verts {
            min = min.min(*v);
            max = max.max(*v);
        }
        // 混合结果可能超出顶点 bbox（曲线外凸），再加一点余量
        let margin = (max - min).max_element() * 0.15 + 3.0;
        min -= margin;
        max += margin;

        let mut grid = VoxelGrid::new(min, max);
        let spacing = grid.voxel * 0.45;
        log!(
            "brute rebuild: {} colors, grid {:?}, voxel {:.2}",
            n,
            grid.dims,
            grid.voxel
        );

        // 2. 采样并打点
        let mut samples: Vec<Vec3> = verts.clone();
        for i in 0..n {
            for j in i + 1..n {
                sample_pair(&latents[i], &latents[j], spacing, &mut samples);
            }
        }
        for i in 0..n {
            for j in i + 1..n {
                for k in j + 1..n {
                    sample_triple(&latents[i], &latents[j], &latents[k], spacing, &mut samples);
                }
            }
        }
        if n >= 4 {
            let mut rng = Rng::new();
            sample_random(&latents, RANDOM_SAMPLES, &mut rng, &mut samples);
        }
        log!("  {} samples", samples.len());

        for p in &samples {
            grid.stamp(*p);
        }

        // 3. 闭运算 + 实心化
        grid.close();
        grid.solidify();

        // 4. 提取边界
        let (mut positions, indices) = boundary_quads(&grid);
        log!(
            "  surface nets: {} verts, {} tris",
            positions.len(),
            indices.len() / 3
        );
        if positions.is_empty() {
            return;
        }
        smooth(&mut positions, &indices, SMOOTH_ITERS);

        mesh.add(&positions, &indices);
    }
}

impl Hull for BruteHull {
    fn new(colors: Vec<Rgb>) -> Result<Box<Self>, BoxError> {
        let mut hull = BruteHull {
            colors,
            mesh: Arc::new(Mutex::new(Mesh::new())),
        };
        hull.rebuild();
        Ok(Box::new(hull))
    }

    fn insert(&mut self, color: Rgb) {
        self.colors.push(color);
        self.rebuild();
    }

    fn mesh(&self) -> Arc<Mutex<Mesh>> {
        self.mesh.clone()
    }
}

// ── 单测 ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::BruteHull;
    use crate::hull::Hull;
    use std::collections::HashMap;

    /// 水密自检：每条有向边恰好出现 1 次（等价于每条无向边被 2 个三角形
    /// 以相反方向共享）。这同时验证闭合性和定向一致性。
    fn assert_watertight(hull: &BruteHull) {
        let mesh = hull.mesh.lock().unwrap();
        let indices = &mesh.indices;
        assert!(!indices.is_empty(), "empty mesh");

        let mut directed: HashMap<(u32, u32), usize> = HashMap::new();
        for tri in indices.chunks(3) {
            for k in 0..3 {
                let a = tri[k];
                let b = tri[(k + 1) % 3];
                assert_ne!(a, b, "degenerate edge in triangle {:?}", tri);
                *directed.entry((a, b)).or_insert(0) += 1;
            }
        }
        for (&(a, b), &cnt) in &directed {
            assert_eq!(cnt, 1, "directed edge ({},{}) appears {} times", a, b, cnt);
            let rev = directed.get(&(b, a)).copied().unwrap_or(0);
            assert_eq!(rev, 1, "edge ({},{}) has no opposite half-edge", a, b);
        }
    }

    #[test]
    fn test_user_case_5_plus_8() {
        let initial = [0xffffff_u32, 0x231f20, 0xed1c24, 0xffdd00, 0x005aaa];
        let colors: Vec<_> = initial.iter().map(|&h| crate::hex_to_rgb(h)).collect();
        let mut hull = BruteHull::new(colors).unwrap();
        assert_watertight(&hull);

        let inserts = [
            0xff0000_u32,
            0x00ff00,
            0x0000ff,
            0x00ffff,
            0xff00ff,
            0xffff00,
            0x000000,
            0xffffff,
        ];
        for &h in &inserts {
            hull.insert(crate::hex_to_rgb(h));
            assert_watertight(&hull);
        }
    }
}
