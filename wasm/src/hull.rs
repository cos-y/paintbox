use fixedbitset::FixedBitSet;
use glam::Vec3;
use lab::Lab;
use mixbox::{float_rgb_to_latent, latent_to_float_rgb};

use crate::{BoxError, Latent, Rgb};

// ── 参数 ─────────────────────────────────────────────────────────────────────

/// 随机混合样本数
const RANDOM_SAMPLES: usize = if cfg!(test) { 30_000 } else { 50_000 };
const RANDOM_MIX_MAX: usize = 6;

// ── 随机数 ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
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

    fn next_unit(&mut self) -> f32 {
        (self.next_u32() as f64 + 1.0) as f32 / (u32::MAX as f64 + 1.0) as f32
    }

    fn next_below(&mut self, n: usize) -> usize {
        (self.next_u32() as usize) % n
    }
}

// ── 体素栅格 ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct VoxelGrid {
    /// Lab 空间原点对应的网格坐标（可以是负的，内部 padding）
    origin: [i32; 3],
    /// 网格尺寸（每格的 Lab 单位）
    size: f32,
    /// 网格维度 [nx, ny, nz]
    dims: [usize; 3],
    /// 占据位图
    bits: FixedBitSet,
}

impl VoxelGrid {
    /// 从 Lab bbox 创建栅格，自动加 padding
    fn new(min: Vec3, max: Vec3, size: f32) -> Self {
        let pad = 2.0 * size;
        let min_padded = min - pad;
        let max_padded = max + pad;
        let extent = max_padded - min_padded;

        let dims = [
            (extent.x / size).ceil() as usize + 1,
            (extent.y / size).ceil() as usize + 1,
            (extent.z / size).ceil() as usize + 1,
        ];
        let origin = [
            (min_padded.x / size).floor() as i32,
            (min_padded.y / size).floor() as i32,
            (min_padded.z / size).floor() as i32,
        ];

        let bits = FixedBitSet::with_capacity(dims[0] * dims[1] * dims[2]);
        VoxelGrid {
            origin,
            size,
            dims,
            bits,
        }
    }

    #[inline]
    fn world_to_grid(&self, p: Vec3) -> [i32; 3] {
        [
            (p.x / self.size).floor() as i32,
            (p.y / self.size).floor() as i32,
            (p.z / self.size).floor() as i32,
        ]
    }

    #[inline]
    fn grid_to_index(&self, gx: i32, gy: i32, gz: i32) -> Option<usize> {
        let lx = (gx - self.origin[0]) as usize;
        let ly = (gy - self.origin[1]) as usize;
        let lz = (gz - self.origin[2]) as usize;
        if lx < self.dims[0] && ly < self.dims[1] && lz < self.dims[2] {
            Some((lz * self.dims[1] + ly) * self.dims[0] + lx)
        } else {
            None
        }
    }

    fn stamp(&mut self, p: Vec3) {
        let g = self.world_to_grid(p);
        if let Some(i) = self.grid_to_index(g[0], g[1], g[2]) {
            self.bits.insert(i);
        }
    }

    fn get(&self, lx: usize, ly: usize, lz: usize) -> bool {
        let i = (lz * self.dims[1] + ly) * self.dims[0] + lx;
        self.bits.contains(i)
    }

    /// 形态学闭运算
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
                    let i = (z * ny + y) * nx + x;
                    if src.contains(i) {
                        if x > 0 {
                            out.insert(i - 1);
                        }
                        if x + 1 < nx {
                            out.insert(i + 1);
                        }
                        if y > 0 {
                            out.insert(i - nx);
                        }
                        if y + 1 < ny {
                            out.insert(i + nx);
                        }
                        if z > 0 {
                            out.insert(i - nx * ny);
                        }
                        if z + 1 < nz {
                            out.insert(i + nx * ny);
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
                    let i = (z * ny + y) * nx + x;
                    let keep = src.contains(i)
                        && (x == 0 || src.contains(i - 1))
                        && (x + 1 >= nx || src.contains(i + 1))
                        && (y == 0 || src.contains(i - nx))
                        && (y + 1 >= ny || src.contains(i + nx))
                        && (z == 0 || src.contains(i - nx * ny))
                        && (z + 1 >= nz || src.contains(i + nx * ny));
                    if keep {
                        out.insert(i);
                    }
                }
            }
        }
        out
    }

    /// 泛洪 solidify：从外边界标记外部，取反得实心体
    fn solidify(&mut self) {
        let [nx, ny, nz] = self.dims;
        let mut outside = FixedBitSet::with_capacity(self.bits.len());
        let mut queue: Vec<(usize, usize, usize)> = vec![];

        let mut try_push = |x: usize, y: usize, z: usize| {
            let i = (z * ny + y) * nx + x;
            if !self.bits.contains(i) && !outside.contains(i) {
                outside.insert(i);
                queue.push((x, y, z));
            }
        };

        // 六个面种子
        for z in 0..nz {
            for y in 0..ny {
                try_push(0, y, z);
                try_push(nx - 1, y, z);
            }
        }
        for z in 0..nz {
            for x in 0..nx {
                try_push(x, 0, z);
                try_push(x, ny - 1, z);
            }
        }
        for y in 0..ny {
            for x in 0..nx {
                try_push(x, y, 0);
                try_push(x, y, nz - 1);
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

        let mut solid = FixedBitSet::with_capacity(self.bits.len());
        for i in 0..self.bits.len() {
            if !outside.contains(i) {
                solid.insert(i);
            }
        }
        self.bits = solid;
    }

    /// 提取实心体素的网格坐标（去除 padding 偏移，返回用户坐标系）
    fn extract_voxels(&self) -> Vec<[i32; 3]> {
        let [nx, ny, nz] = self.dims;
        let mut out = Vec::new();
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let i = (z * ny + y) * nx + x;
                    if self.bits.contains(i) {
                        // 转回世界坐标系（带 origin 偏移）
                        let gx = self.origin[0] + x as i32;
                        let gy = self.origin[1] + y as i32;
                        let gz = self.origin[2] + z as i32;
                        out.push([gx, gy, gz]);
                    }
                }
            }
        }
        out
    }
}

// ── 采样 ─────────────────────────────────────────────────────────────────────

fn latent_to_lab_vec(latent: &Latent) -> Vec3 {
    let rgb = latent_to_float_rgb(latent);
    let lab = Lab::from_rgb_normalized(&rgb);
    Vec3::new(lab.l, lab.a, lab.b)
}

fn sample_pair(a: &Latent, b: &Latent, spacing: f32, out: &mut Vec<Vec3>) {
    let la = latent_to_lab_vec(a);
    let lb = latent_to_lab_vec(b);
    let n = ((la.distance(lb) / spacing).ceil() as usize).max(2);
    for k in 0..=n {
        let t = k as f32 / n as f32;
        let l: Latent = std::array::from_fn(|i| (1.0 - t) * a[i] + t * b[i]);
        out.push(latent_to_lab_vec(&l));
    }
}

fn sample_triple(a: &Latent, b: &Latent, c: &Latent, spacing: f32, out: &mut Vec<Vec3>) {
    let la = latent_to_lab_vec(a);
    let lb = latent_to_lab_vec(b);
    let lc = latent_to_lab_vec(c);
    let ext = la.distance(lb).max(lb.distance(lc)).max(la.distance(lc));
    // triple 用更宽松的 spacing（patch 内部不需要太密，主要靠 pair 边界 + random 填充）
    let triple_spacing = spacing * 2.2; // 原来是 0.45*grid_size，现在是 1.0*grid_size
    let n = ((ext / triple_spacing).ceil() as usize).max(2);
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

fn sample_random(latents: &[Latent], count: usize, rng: &mut Rng, out: &mut Vec<Vec3>) {
    let n = latents.len();
    if n < 2 {
        return;
    }
    let mut idx = [0usize; RANDOM_MIX_MAX];
    let mut w = [0f32; RANDOM_MIX_MAX];
    for _ in 0..count {
        let k = 2 + rng.next_below(RANDOM_MIX_MAX.min(n) - 1);
        let mut sum = 0f32;
        for s in 0..k {
            idx[s] = rng.next_below(n);
            let e = -rng.next_unit().ln();
            w[s] = e;
            sum += e;
        }
        let l: Latent =
            std::array::from_fn(|d| (0..k).map(|s| latents[idx[s]][d] * w[s] / sum).sum());
        out.push(latent_to_lab_vec(&l));
    }
}

// ── Hull ──────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct Hull {
    grid_size: f32,
    /// 输入颜色的 latent
    latents: Vec<Latent>,
    /// 输入颜色的 Lab（用于 bbox）
    labs: Vec<Vec3>,
    /// 体素栅格（增量维护，累积打点）
    grid: Option<VoxelGrid>,
    /// 随机数生成器
    rng: Rng,

    /// 输出：体素网格坐标 [x,y,z, x,y,z, ...]
    pub indices: Vec<i32>,
    /// 输出：体素颜色 [r,g,b, r,g,b, ...] (0-1 归一化 sRGB)
    pub colors: Vec<i32>,
}

impl Hull {
    pub fn new(grid_size: f32, colors: Vec<Rgb>) -> Result<Self, BoxError> {
        // 固定全色域栅格，原点锚定（坐标跨 insert 稳定）
        // Lab 范围：L 0..100, a/b -128..128，各方向留 padding
        let grid = VoxelGrid::new(
            Vec3::new(-4.0, -132.0, -132.0),
            Vec3::new(104.0, 132.0, 132.0),
            grid_size,
        );
        let mut hull = Hull {
            grid_size,
            latents: Vec::new(),
            labs: Vec::new(),
            grid: Some(grid),
            rng: Rng::new(),
            indices: Vec::new(),
            colors: Vec::new(),
        };
        for c in colors {
            hull.insert(c);
        }
        hull.finalize();
        Ok(hull)
    }

    /// 增量插入：只采样“含新颜色”的 pair/triple，打点累积进栅格。
    /// 不做 close/solidify/extract（惰性，留到 finalize）。
    pub fn insert(&mut self, color: Rgb) {
        let latent = float_rgb_to_latent(&color);
        let lab = latent_to_lab_vec(&latent);
        let new_idx = self.latents.len();
        self.latents.push(latent);
        self.labs.push(lab);

        let spacing = self.grid_size * 0.45;
        let grid = self.grid.as_mut().unwrap();

        // 新颜色自身
        grid.stamp(lab);

        // 含新颜色的 pair：(new, j)
        let mut buf: Vec<Vec3> = Vec::new();
        for j in 0..new_idx {
            sample_pair(&self.latents[new_idx], &self.latents[j], spacing, &mut buf);
        }
        // O(n³) triple 采样移除，用 random 高阶混合填充内部体积
        // for j in 0..new_idx {
        //     for k in j + 1..new_idx {
        //         sample_triple(
        //             &self.latents[new_idx],
        //             &self.latents[j],
        //             &self.latents[k],
        //             spacing,
        //             &mut buf,
        //         );
        //     }
        // }
        for &p in &buf {
            grid.stamp(p);
        }

        // 含新颜色的随机高阶混合（约束到必含 new_idx）
        if self.latents.len() >= 4 {
            let n = self.latents.len();
            let mut idx = [0usize; RANDOM_MIX_MAX];
            let mut w = [0f32; RANDOM_MIX_MAX];
            for _ in 0..RANDOM_SAMPLES {
                let k = 2 + self.rng.next_below(RANDOM_MIX_MAX.min(n) - 1);
                idx[0] = new_idx; // 保证含新颜色
                let mut sum;
                {
                    let e = -self.rng.next_unit().ln();
                    w[0] = e;
                    sum = e;
                }
                for s in 1..k {
                    idx[s] = self.rng.next_below(n);
                    let e = -self.rng.next_unit().ln();
                    w[s] = e;
                    sum += e;
                }
                let l: Latent = std::array::from_fn(|d| {
                    (0..k).map(|s| self.latents[idx[s]][d] * w[s] / sum).sum()
                });
                grid.stamp(latent_to_lab_vec(&l));
            }
        }

        self.finalize();
    }

    /// 在累积栅格副本上跑 close/solidify/extract，生成体素输出。
    fn finalize(&mut self) {
        let raw = self.grid.as_ref().unwrap();
        if self.latents.is_empty() {
            self.indices.clear();
            self.colors.clear();
            return;
        }

        // 在副本上做形态学与实心化，raw 保留累积打点
        let mut work = raw.clone();
        work.close();
        work.solidify();

        let voxels = work.extract_voxels();
        self.indices.clear();
        self.colors.clear();
        self.indices.reserve(voxels.len() * 3);
        self.colors.reserve(voxels.len() * 3);

        for &[gx, gy, gz] in &voxels {
            self.indices.push(gx);
            self.indices.push(gy);
            self.indices.push(gz);

            // 体素中心 Lab 直接转 sRGB（O(1)，不再扫所有颜色）
            let l = (gx as f32 + 0.5) * self.grid_size;
            let a = (gy as f32 + 0.5) * self.grid_size;
            let b = (gz as f32 + 0.5) * self.grid_size;
            let [r, g, b] = Lab { l, a, b }.to_rgb_normalized();
            let r = ((255.0 * r).round() as i32).clamp(0, 255);
            let g = ((255.0 * g).round() as i32).clamp(0, 255);
            let b = ((255.0 * b).round() as i32).clamp(0, 255);
            self.colors.push((r << 16) | (g << 8) | b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let colors = vec![
            crate::hex_to_rgb(0xffffff),
            crate::hex_to_rgb(0x000000),
            crate::hex_to_rgb(0xff0000),
        ];
        let hull = Hull::new(2.0, colors).unwrap();
        assert!(!hull.indices.is_empty());
        assert_eq!(hull.indices.len() / 3, hull.colors.len() / 3);
    }
}

#[cfg(test)]
mod bench {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_incremental() {
        let seq = [
            0xff0000u32,
            0x00ff00,
            0x0000ff,
            0x00ffff,
            0xff00ff,
            0xffff00,
            0x000000,
            0xffffff,
        ];
        for &gs in &[3.0f32, 2.0] {
            let t0 = Instant::now();
            let mut hull = Hull::new(gs, vec![]).unwrap();
            for &h in &seq {
                hull.insert(crate::hex_to_rgb(h));
            }
            let t_add = t0.elapsed();
            let t1 = Instant::now();
            let nv = hull.indices.len() / 3;
            let t_fin = t1.elapsed();
            eprintln!(
                "gs={} add={:?} finalize={:?} voxels={}",
                gs, t_add, t_fin, nv
            );
        }
    }

    #[test]
    fn bench_24colors() {
        let colors = (0..24).map(|i| {
            let h = i as f32 / 24.0;
            let r = ((h * 6.0).sin() * 127.0 + 128.0) as u8;
            let g = ((h * 6.0 + 2.0).sin() * 127.0 + 128.0) as u8;
            let b = ((h * 6.0 + 4.0).sin() * 127.0 + 128.0) as u8;
            [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
        }).collect::<Vec<_>>();

        // 统计采样点数
        let latents: Vec<_> = colors.iter().map(|c| mixbox::float_rgb_to_latent(c)).collect();
        let spacing = 10.0 * 0.45;
        let mut pair_count = 0;
        let mut triple_count = 0;
        
        for i in 0..24 {
            for j in i+1..24 {
                let la = latent_to_lab_vec(&latents[i]);
                let lb = latent_to_lab_vec(&latents[j]);
                let n = ((la.distance(lb) / spacing).ceil() as usize).max(2);
                pair_count += n + 1;
            }
        }
        
        for i in 0..24 {
            for j in i+1..24 {
                for k in j+1..24 {
                    let la = latent_to_lab_vec(&latents[i]);
                    let lb = latent_to_lab_vec(&latents[j]);
                    let lc = latent_to_lab_vec(&latents[k]);
                    let ext = la.distance(lb).max(lb.distance(lc)).max(la.distance(lc));
                    let n = ((ext / spacing).ceil() as usize).max(2);
                    let pts = (n+1)*(n+2)/2;
                    triple_count += pts;
                }
            }
        }
        
        eprintln!("samples: pairs={}, triples={}, random={}, total={}", 
            pair_count, triple_count, RANDOM_SAMPLES, pair_count + triple_count + RANDOM_SAMPLES);

        let t0 = Instant::now();
        let hull = Hull::new(10.0, colors).unwrap();
        let t_new = t0.elapsed();
        eprintln!("24 colors, gs=10: total={:?}, voxels={}", t_new, hull.indices.len()/3);
    }
}

#[test]
fn bench_24colors() {
    let colors = (0..24).map(|i| {
        let h = i as f32 / 24.0;
        let r = ((h * 6.0).sin() * 127.0 + 128.0) as u8;
        let g = ((h * 6.0 + 2.0).sin() * 127.0 + 128.0) as u8;
        let b = ((h * 6.0 + 4.0).sin() * 127.0 + 128.0) as u8;
        [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
    }).collect::<Vec<_>>();

    let t0 = std::time::Instant::now();
    let hull = Hull::new(10.0, colors).unwrap();
    let t_new = t0.elapsed();
    eprintln!("new: {:?}, voxels: {}", t_new, hull.indices.len()/3);
}

    #[test]
    fn count_samples() {
        let colors = (0..24).map(|i| {
            let h = i as f32 / 24.0;
            let r = ((h * 6.0).sin() * 127.0 + 128.0) as u8;
            let g = ((h * 6.0 + 2.0).sin() * 127.0 + 128.0) as u8;
            let b = ((h * 6.0 + 4.0).sin() * 127.0 + 128.0) as u8;
            [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
        }).collect::<Vec<_>>();

        let latents: Vec<_> = colors.iter().map(|c| mixbox::float_rgb_to_latent(c)).collect();
        let spacing = 10.0 * 0.45;
        let mut pair_count = 0;
        let mut triple_count = 0;
        
        for i in 0..24 {
            for j in i+1..24 {
                let la = latent_to_lab_vec(&latents[i]);
                let lb = latent_to_lab_vec(&latents[j]);
                let n = ((la.distance(lb) / spacing).ceil() as usize).max(2);
                pair_count += n + 1;
            }
        }
        
        for i in 0..24 {
            for j in i+1..24 {
                for k in j+1..24 {
                    let la = latent_to_lab_vec(&latents[i]);
                    let lb = latent_to_lab_vec(&latents[j]);
                    let lc = latent_to_lab_vec(&latents[k]);
                    let ext = la.distance(lb).max(lb.distance(lc)).max(la.distance(lc));
                    let n = ((ext / spacing).ceil() as usize).max(2);
                    let pts = (n+1)*(n+2)/2;
                    triple_count += pts;
                }
            }
        }
        
        eprintln!("pairs: {}, triples: {}, random: {}, total: {}", 
            pair_count, triple_count, 50_000, pair_count + triple_count + 50_000);
    }
