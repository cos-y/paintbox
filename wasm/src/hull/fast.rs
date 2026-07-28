use core::f32;
use std::{
    fmt::{Debug, Display},
    mem::swap,
    sync::{Arc, Mutex},
};

use derivative::Derivative;
use empfindung::ToLab;
use fixedbitset::FixedBitSet;
use glam::Vec3;
use lab::Lab;
use mixbox::{float_rgb_to_latent, latent_to_float_rgb};
use ordered_float::OrderedFloat;

use crate::{BoxError, Latent, Rgb, hull::Hull, log, mesh::Mesh, tess::get_triangle_tesselation};

fn latent_to_lab(latent: &Latent) -> Lab {
    let rgb = latent_to_float_rgb(&latent);
    Lab::from_rgb_normalized(&rgb)
}

fn lab_to_vec3(lab: &Lab) -> Vec3 {
    Vec3::new(lab.l, lab.a, lab.b)
}

fn sample_microsphere(p: &Lab, eps: f32) -> Lab {
    let mut buf = [0u8; 3];
    getrandom::fill(&mut buf).unwrap();
    // [-eps, eps]
    // 采样不均匀没关系
    let [dl, da, db] = buf.map(|x| x as f32 * eps / 255f32 * 2f32 - 1f32);
    Lab {
        l: (p.l + dl).clamp(0f32, 100f32),
        a: (p.a + da).clamp(-125f32, 125f32),
        b: (p.b + db).clamp(-125f32, 125f32),
    }
}

// 估算曲线ab需要的细分次数
fn get_subdivision_count(a: &Lab, b: &Lab) -> usize {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    let d = (dl * dl + da * da + db * db).sqrt();
    (d / 4f32).ceil() as usize
    // 1
}

#[derive(Debug)]
struct Facet {
    // 三角曲面的三个顶点，ccw
    vs: [usize; 3],
    // 每个顶点对边的邻接面的下标，用于泛洪更新
    fs: [usize; 3],
}

// #[derive(Debug)]
struct Point {
    // mixbox仿射空间坐标
    latent: Latent,
    // lab空间坐标
    lab: Lab,
    // 原始颜色的下标
    idx: usize,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct FastHull {
    // 原始颜色的srgb
    #[derivative(Debug = "ignore")]
    raw_colors: Vec<Rgb>,
    // 点云
    #[derivative(Debug = "ignore")]
    points: Vec<Point>,
    // 保存所有的面。为空代表已回收，新加入的顶点把它移出凸包了。
    facets: Vec<Option<Facet>>,
    // 已经被回收的面的下标。当我们要新分配一个面时，优先查找并更新这个表，以避免频繁内存分配。
    null_facets: Vec<usize>,
    // 重心的隐空间坐标
    #[derivative(Debug = "ignore")]
    center_latent: Latent,
    // 重心的lab空间坐标
    center: Vec3,

    pub mesh: Arc<Mutex<Mesh>>,
}

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "({},{},{})",
            self.lab.l, self.lab.a, self.lab.b,
        ))
    }
}

// impl Display for Facet {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_fmt(format_args!(
//             "<{},{},{}>",
//             self.vs[0], self.vs[1], self.vs[2],
//         ))
//     }
// }

impl Display for Facet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<{},{},{}>::[{},{},{}]",
            self.vs[0], self.vs[1], self.vs[2], self.fs[0], self.fs[1], self.fs[2],
        ))
    }
}

impl Display for FastHull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct FacetList<'a>(&'a [Option<Facet>]);

        impl<'a> std::fmt::Debug for FacetList<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut list = f.debug_list();
                for facet in self.0 {
                    match facet {
                        // format_args! 实现了 Debug，可以直接调用 facet 的 Display 输出
                        Some(f) => list.entry(&format_args!("{f}")),
                        // 零分配输出 `-`（若需要带双引号的 "-"，写 &"-" 即可）
                        None => list.entry(&format_args!("_")),
                    };
                }
                list.finish()
            }
        }

        f.debug_struct("Hull")
            .field("facets", &FacetList(&self.facets))
            // .field("other_field", &self.other_field)
            .finish()
    }
}

impl Facet {
    pub fn reroute_f(&mut self, old: usize, new: usize) {
        if old != new {
            if let Some(i) = self.fs.iter_mut().find(|x| **x == old) {
                *i = new;
            }
        }
    }
}

impl Hull for FastHull {
    fn new(raw_colors: Vec<Rgb>) -> Result<Box<FastHull>, BoxError> {
        if raw_colors.is_empty() {
            todo!();
        }

        let mut points: Vec<_> = raw_colors
            .iter()
            .enumerate()
            .map(|(idx, rgb)| Point {
                latent: float_rgb_to_latent(&rgb),
                lab: Lab::from_rgb_normalized(&rgb),
                idx,
            })
            .collect();

        // Point不一定等于用户输入的原始颜色，假如用户只输入了1-3个颜色，我们需要在原始颜色的微小邻域内采样一些phantom point，让输入至少能构成一个四面体。
        // 我们让phantom point的idx指向对应原始颜色。
        while points.len() < 4 {
            let mut buf = [0u8; 1];
            getrandom::fill(&mut buf).unwrap();
            let idx = buf[0] as usize % raw_colors.len();
            let lab = sample_microsphere(&points[idx].lab, 1e-3);
            // li.push(vertex!(coord, data));
            let rgb = lab.to_rgb_normalized();
            points.push(Point {
                latent: float_rgb_to_latent(&rgb),
                lab,
                idx,
            });
        }

        fn find_minmax(li: impl Iterator<Item = (usize, f32)>) -> (usize, usize) {
            let (mut minx, mut maxx) = (f32::MAX, f32::MIN);
            let (mut mini, mut maxi) = (usize::MAX, usize::MAX);
            for (i, x) in li {
                if x < minx {
                    (mini, minx) = (i, x);
                }
                if x > maxx {
                    (maxi, maxx) = (i, x);
                }
            }
            (mini, maxi)
        }

        // 我们希望后续更新尽可能少，所以应该让初始四面体覆盖尽可能大的色域。
        // 根据经验，我们取l最小和最大的两个点，以及除了这两个点之外ab最小和最大的两个点。
        let (v0, v1) = find_minmax(points.iter().map(|p| p.lab.l).enumerate());
        // 总不可能所有的l值都一样吧。
        let (v0, v1) = if v0 == v1 { (0, 1) } else { (v0, v1) };

        let (v2, v3) = find_minmax(points.iter().enumerate().filter_map(|(v, p)| {
            if v == v0 || v == v1 {
                None
            } else {
                Some((v, p.lab.a + p.lab.b))
            }
        }));
        let (v2, v3) = if v2 == v3 {
            let v3 = (0..points.len())
                .find(|v| ![v0, v1, v2].contains(v))
                .unwrap();
            (v2, v3)
        } else {
            (v2, v3)
        };

        let vs0 = [v0, v1, v2, v3];
        log!("{:?}", vs0);

        // 控制顶点的添加顺序，我们希望优先添加最有希望更新凸包的顶点，以最小化泛洪更新的次数
        // 先排除初始四个顶点
        let mut vs: Vec<_> = (0..points.len()).filter(|v| !vs0.contains(v)).collect();
        let ws: Vec<_> = points
            .iter()
            .map(|p| {
                let (l, a, b) = p.lab.to_lab();
                // 目测表明，l轴更有可能改变混合结果
                l.abs() * 2f32 + a.abs() + b.abs()
            })
            .collect();
        vs.sort_by_key(|v| OrderedFloat(ws[*v]));

        let center_latent: Latent =
            std::array::from_fn(|i| vs0.iter().map(|v| points[*v].latent[i] * 0.25).sum());
        let center = latent_to_float_rgb(&center_latent);
        let center = Lab::from_rgb_normalized(&center);
        let center = Vec3::new(center.l, center.a, center.b);

        log!(":: points :: {:?}", points);

        let mut hull = FastHull {
            raw_colors,
            points,
            facets: vec![],
            null_facets: vec![],
            center_latent,
            center,
            mesh: Arc::new(Mutex::new(Mesh::new())),
        };

        // log!("{}", hull.center);

        // 构造初始四面体
        for vp in vs0.into_iter() {
            let mut vs = [0usize; 3];
            let mut fs = [0usize; 3];
            let iter = vs0.into_iter().enumerate().filter(|(_, v)| *v != vp);
            for (j, (i, v)) in iter.enumerate() {
                vs[j] = v;
                fs[j] = i;
            }

            // 应用mixbox的重心必在凸包内侧，根据这个性质来检测每个面是否需要反转。我们需要保证每个面的绕行顺序都是ccw。
            if hull.get_facet_side(&hull.center_latent, vs) > 0.0 {
                (vs[1], vs[2]) = (vs[2], vs[1]);
                (fs[1], fs[2]) = (fs[2], fs[1]);
            }

            hull.facets.push(Some(Facet { vs, fs }));
        }

        log!(":: init :: {}", hull);

        for v in vs {
            hull.do_insert_point(v);
        }

        hull.build_mesh();

        // log!(":: new :: {}", hull);

        Ok(Box::new(hull))
    }

    fn insert(&mut self, color: Rgb) {
        let idx = self.raw_colors.len();
        self.raw_colors.push(color);

        let point = Point {
            latent: float_rgb_to_latent(&color),
            lab: Lab::from_rgb_normalized(&color),
            idx,
        };
        let idx = self.points.len();
        self.points.push(point);

        log!(":: points :: {:?}", self.points);

        if self.do_insert_point(idx) {
            self.build_mesh();
        }
    }

    fn mesh(&self) -> Arc<Mutex<Mesh>> {
        self.mesh.clone()
    }
}

impl FastHull {
    /// 修复凸性违规：扫描并重新插入跑出面外侧的点
    fn repair_convexity(&mut self) {
        let max_rounds = 10;
        for _round in 0..max_rounds {
            let mut fixed = false;
            for v in 0..self.points.len() {
                for fi in self.facets.iter() {
                    let fi = match fi {
                        Some(f) => f,
                        None => continue,
                    };
                    if fi.vs.contains(&v) {
                        continue;
                    }
                    let side = self.get_facet_side(&self.points[v].latent, fi.vs);
                    if side > 0.0 {
                        log!(
                            ":: repair: point {} outside face [{} {} {}]",
                            v,
                            fi.vs[0],
                            fi.vs[1],
                            fi.vs[2]
                        );
                        self.do_insert_point(v);
                        fixed = true;
                        break;
                    }
                }
                if fixed {
                    break;
                }
            }
            if !fixed {
                break;
            }
        }
    }

    fn build_mesh(&mut self) {
        let mut mesh = self.mesh.lock().unwrap();
        mesh.clear();
        for facet in self.facets.iter() {
            if let Some(facet) = facet {
                let (vs, is) = self.build_facet_mesh(facet.vs);
                mesh.add(&vs, &is);
            }
        }
    }

    fn build_facet_mesh(&self, vs: [usize; 3]) -> (Vec<Vec3>, Arc<Vec<usize>>) {
        let labs = vs.map(|v| &self.points[v].lab);
        let latents = vs.map(|v| &self.points[v].latent);

        let ks = [
            get_subdivision_count(&labs[1], &labs[2]),
            get_subdivision_count(&labs[0], &labs[2]),
            get_subdivision_count(&labs[0], &labs[1]),
        ];

        let tess = get_triangle_tesselation(&ks);
        let vertices = tess
            .uvs
            .iter()
            .map(|(t2, t1)| {
                let latent: Latent = std::array::from_fn(|i| {
                    t1 * latents[1][i] + t2 * latents[2][i] + (1.0 - t1 - t2) * latents[0][i]
                });
                let rgb = latent_to_float_rgb(&latent);
                let (l, a, b) = Lab::from_rgb_normalized(&rgb).to_lab();
                Vec3::new(l, a, b)
            })
            .collect();

        (vertices, tess.triangles.clone())
    }

    // 检测点p在面vs的哪一侧
    // 返回 -1/0/+1：p在面内侧(-1)、在面上(0)、在面外侧(+1)
    //
    // 方法：在 latent 空间中，4个点 (a,b,c,p) 张成一个 3D 仿射子空间。
    // 选取 latent 坐标维度做投影，计算 3×3 行列式，
    // 其符号 = 四点单纯形在该投影下的有向体积符号。
    fn get_facet_side(&self, p: &Latent, vs: [usize; 3]) -> f32 {
        let [a, b, c] = vs;
        let pa = &self.points[a].latent;
        let pb = &self.points[b].latent;
        let pc = &self.points[c].latent;

        // 如果 p 与某个面顶点重合，直接返回 0
        for &pt in &[pa, pb, pc] {
            let d2: f32 = (0..7)
                .map(|i| {
                    let d = p[i] - pt[i];
                    d * d
                })
                .sum();
            if d2 < 1e-16 {
                return 0.0;
            }
        }

        let v1: [f32; 7] = std::array::from_fn(|i| pb[i] - pa[i]);
        let v2: [f32; 7] = std::array::from_fn(|i| pc[i] - pa[i]);
        let v3: [f32; 7] = std::array::from_fn(|i| p[i] - pa[i]);

        #[inline]
        fn det3(a: &[f32; 7], b: &[f32; 7], c: &[f32; 7], i0: usize, i1: usize, i2: usize) -> f32 {
            a[i0] * (b[i1] * c[i2] - b[i2] * c[i1]) - a[i1] * (b[i0] * c[i2] - b[i2] * c[i0])
                + a[i2] * (b[i0] * c[i1] - b[i1] * c[i0])
        }

        const DIMS: [(usize, usize, usize); 5] =
            [(0, 1, 2), (0, 1, 3), (0, 2, 3), (1, 2, 3), (0, 1, 4)];

        // 固定优先用 (0,1,2)，退化时依次回退。同一坐标三元组对所有测试方向一致。
        for &(i0, i1, i2) in &DIMS {
            let d = det3(&v1, &v2, &v3, i0, i1, i2);
            if d.abs() > 1e-8 {
                return d.signum();
            }
        }

        // 所有 latent 投影退化 → 回退到 lab 空间有向体积
        let p_lab = lab_to_vec3(&latent_to_lab(p));
        let a_lab = lab_to_vec3(&self.points[a].lab);
        let b_lab = lab_to_vec3(&self.points[b].lab);
        let c_lab = lab_to_vec3(&self.points[c].lab);
        let n_lab = (b_lab - a_lab).cross(c_lab - a_lab);
        let d = n_lab.dot(p_lab - a_lab);
        if d.abs() < 1e-4 {
            return 0.0;
        }
        d.signum()
    }

    fn try_fold_facet(&mut self, i: usize) -> bool {
        let facet = self.facets[i].as_ref();
        if facet.is_none() {
            return false;
        }

        let fs: [usize; 3] = facet.unwrap().fs;

        let mut do_fold = |j: usize, x: usize| {
            let facet = self.facets[j].as_ref().unwrap();
            let y = facet.fs.into_iter().find(|x| *x != i).unwrap();

            log!(
                ":: fold {} - {}",
                self.facets[i].as_ref().unwrap(),
                self.facets[j].as_ref().unwrap()
            );
            self.deallocate_facet(i);
            self.deallocate_facet(j);

            let fx = self.facets[x].as_mut().unwrap();
            let rx = fx.fs.iter_mut().find(|x| **x == i).unwrap();
            *rx = y;

            let fy = self.facets[y].as_mut().unwrap();
            let ry = fy.fs.iter_mut().find(|x| **x == j).unwrap();
            *ry = x;

            (x, y)
        };

        let (x, y) = match fs {
            [a, b, c] if a == b => do_fold(a, c),
            [a, b, c] if b == c => do_fold(b, a),
            [a, b, c] if c == a => do_fold(c, b),
            _ => return false,
        };

        self.try_fold_facet(x);
        self.try_fold_facet(y);

        true
    }

    fn do_insert_point(&mut self, v: usize) -> bool {
        log!("insert :: {}", &self.points[v].idx);
        // let Point { lab, .. } = &self.points[v];
        // let point = Vec3::new(lab.l, lab.a, lab.b);
        let latent = &self.points[v].latent;

        let facet = self
            .facets
            .iter()
            .enumerate()
            .find(|(_, facet)| {
                facet
                    .as_ref()
                    .map_or(false, |facet| self.get_facet_side(latent, facet.vs) > 0.0)
            })
            .map(|(i, _)| i);

        if let Some(i) = facet {
            self.remesh_facet(i, v);
            log!("{}", self);
            #[cfg(test)]
            self.assert_watertight();
            return true;
        }

        false
    }

    fn allocate_facet(&mut self) -> usize {
        if self.null_facets.is_empty() {
            let idx = self.facets.len();
            self.facets.push(None);
            idx
        } else {
            self.null_facets.swap_remove(0)
        }
    }

    fn deallocate_facet(&mut self, i: usize) -> Facet {
        self.null_facets.push(i);
        let mut facet = None;
        swap(&mut facet, &mut self.facets[i]);
        facet.unwrap()
    }

    fn remesh_facet(&mut self, i: usize, v: usize) {
        let mut visited = FixedBitSet::with_capacity(self.facets.len());

        let facet = self.deallocate_facet(i);
        let i0 = self.allocate_facet();
        let i1 = self.allocate_facet();
        let i2 = self.allocate_facet();

        // ccw
        let f0 = Facet {
            vs: [v, facet.vs[1], facet.vs[2]],
            fs: [facet.fs[0], i1, i2],
        };
        let f1 = Facet {
            vs: [v, facet.vs[2], facet.vs[0]],
            fs: [facet.fs[1], i2, i0],
        };
        let f2 = Facet {
            vs: [v, facet.vs[0], facet.vs[1]],
            fs: [facet.fs[2], i0, i1],
        };

        log!(":: remesh :: {} => {} {} {}", facet, f0, f1, f2);

        self.facets[i0] = Some(f0);
        self.facets[i1] = Some(f1);
        self.facets[i2] = Some(f2);

        // 更新没被删除的半边
        self.facets[facet.fs[0]].as_mut().unwrap().reroute_f(i, i0);
        self.facets[facet.fs[1]].as_mut().unwrap().reroute_f(i, i1);
        self.facets[facet.fs[2]].as_mut().unwrap().reroute_f(i, i2);

        for i in [i0, i1, i2] {
            if i < visited.len() {
                visited.set(i, true);
            }
        }

        log!(":: {}", self);

        for i in [i0, i1, i2] {
            self.remesh_facet_adj_flood(i, 0, &mut visited);
        }
    }

    fn remesh_facet_adj_flood(&mut self, i: usize, vi_idx: usize, visited: &mut FixedBitSet) {
        // assert_watertight(self);
        let fi = self.facets[i].as_ref();
        // 有可能前面的轮次已经把这个面fold掉了，所以这里可以early exit
        if fi.is_none() {
            return;
        }

        let fi = fi.unwrap();
        let j = fi.fs[vi_idx];

        if j >= visited.len() || visited[j] {
            return;
        }

        visited.set(j, true);

        // 先找出待优化四面体的四个顶点
        let va_i_idx = (vi_idx + 1) % 3;
        let vb_i_idx = (vi_idx + 2) % 3;

        let vi = fi.vs[vi_idx];
        let va = fi.vs[va_i_idx];
        let vb = fi.vs[vb_i_idx];

        let fj = self.facets[j].as_ref().unwrap();
        let vj_idx = fj
            .vs
            .into_iter()
            .enumerate()
            .find(|(_, v)| *v != va && *v != vb)
            .map(|(i, _)| i)
            .unwrap();

        let vb_j_idx = (vj_idx + 1) % 3;
        let va_j_idx = (vj_idx + 2) % 3;

        let vj = fj.vs[vj_idx];

        let latents = [vi, va, vb, vj].map(|v| &self.points[v].latent);
        let latent_c: Latent = std::array::from_fn(|i| latents.iter().map(|l| l[i] * 0.25).sum());
        log!(
            ":: in tetrahedron :: [{},{},{},{}] :: m={}",
            vi,
            va,
            vb,
            vj,
            lab_to_vec3(&latent_to_lab(&latent_c))
        );

        // 重心在fi的外面，说明fi失效了，显然fj也失效了，需要做remesh。
        let mut s = self.get_facet_side(&latent_c, fi.vs);
        if s.abs() < 1e-4 {
            s = self.get_facet_side(&latent_c, fj.vs);
            if s.abs() < 1e-4 {
                return;
            }
        }

        if s > 0.0 {
            let fi = self.deallocate_facet(i);
            let fj = self.deallocate_facet(j);

            let i0 = self.allocate_facet();
            let i1 = self.allocate_facet();

            let f0 = Facet {
                vs: [va, vj, vi],
                fs: [i1, fi.fs[vb_i_idx], fj.fs[vb_j_idx]],
            };
            let f1 = Facet {
                vs: [vb, vi, vj],
                fs: [i0, fj.fs[va_j_idx], fi.fs[va_i_idx]],
            };

            log!(":: remesh :: {} {} => {} {}", fi, fj, f0, f1);

            self.facets[i0] = Some(f0);
            self.facets[i1] = Some(f1);

            // 更新没被删除的半边
            self.facets[fi.fs[va_i_idx]]
                .as_mut()
                .unwrap()
                .reroute_f(i, i1);
            self.facets[fi.fs[vb_i_idx]]
                .as_mut()
                .unwrap()
                .reroute_f(i, i0);
            self.facets[fj.fs[va_j_idx]]
                .as_mut()
                .unwrap()
                .reroute_f(j, i1);
            self.facets[fj.fs[vb_j_idx]]
                .as_mut()
                .unwrap()
                .reroute_f(j, i0);

            let i0_folded = self.try_fold_facet(i0);
            let i1_folded = self.try_fold_facet(i1);

            if !i0_folded && i0 < visited.len() {
                visited.set(i0, true);
            }

            if !i1_folded && i1 < visited.len() {
                visited.set(i1, true);
            }

            log!(":: {}", self);

            if !i0_folded {
                self.remesh_facet_adj_flood(i0, 2, visited);
                self.remesh_facet_adj_flood(i0, 1, visited);
            }

            if !i1_folded {
                self.remesh_facet_adj_flood(i1, 1, visited);
                self.remesh_facet_adj_flood(i1, 2, visited);
            }
        }
    }
}

#[cfg(test)]
impl FastHull {
    fn assert_watertight(&self) {
        for (i, fi) in self.facets.iter().enumerate() {
            if let Some(fi) = fi {
                for (vi, fj) in fi.fs.iter().enumerate() {
                    let fj = self.facets[*fj].as_ref().unwrap();
                    let (vj, _) = fj.fs.iter().enumerate().find(|(_, v)| **v == i).unwrap();
                    assert_ne!(fi.vs[vi], fj.vs[vj]);

                    let mut li: Vec<usize> = fi
                        .vs
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != vi)
                        .map(|(_, e)| *e)
                        .collect();
                    li.sort();

                    let mut lj: Vec<usize> = fj
                        .vs
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != vj)
                        .map(|(_, e)| *e)
                        .collect();
                    lj.sort();

                    assert_ne!(li[0], li[1]);
                    assert_ne!(lj[0], lj[1]);
                    assert_eq!(li, lj);
                }
            }
        }
    }

    fn assert_spherical_isoembryonic(&self) {
        use std::collections::HashSet;

        let mut f = 0;
        let mut e = 0;
        let mut s = HashSet::new();
        for facet in self.facets.iter() {
            if let Some(facet) = facet {
                f += 1;
                e += 3;
                for v in facet.vs {
                    s.insert(v);
                }
            }
        }

        let v = s.iter().count();
        e /= 2;

        assert_eq!(f + v - e, 2);

        self.assert_convex();
    }

    /// 验证凸性：对每个面，所有其他顶点都在面的内侧（get_facet_side ≤ 0）
    fn assert_convex(&self) {
        for (i, fi) in self.facets.iter().enumerate() {
            let fi = match fi {
                Some(f) => f,
                None => continue,
            };
            for (v, point) in self.points.iter().enumerate() {
                if fi.vs.contains(&v) {
                    continue;
                }
                let side = self.get_facet_side(&point.latent, fi.vs);
                assert!(
                    side <= 0.0,
                    "facet {} [{},{},{}]: point {} is outside (side={})",
                    i,
                    fi.vs[0],
                    fi.vs[1],
                    fi.vs[2],
                    v,
                    side
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Latent,
        hull::{
            Hull,
            fast::{Facet, FastHull},
        },
        log,
    };

    fn get_facets(hull: &FastHull) -> Vec<[usize; 3]> {
        let mut li: Vec<_> = hull
            .facets
            .iter()
            .filter_map(|x| x.as_ref())
            .map(|Facet { vs, .. }| {
                if vs[0] < vs[1] && vs[0] < vs[2] {
                    *vs
                } else if vs[1] < vs[0] && vs[1] < vs[2] {
                    [vs[1], vs[2], vs[0]]
                } else {
                    [vs[2], vs[0], vs[1]]
                }
            })
            .collect();
        li.sort();
        li
    }

    const R: [f32; 3] = [1f32, 0f32, 0f32];
    const G: [f32; 3] = [0f32, 1f32, 0f32];
    const B: [f32; 3] = [0f32, 0f32, 1f32];
    const C: [f32; 3] = [0f32, 1f32, 1f32];
    const M: [f32; 3] = [1f32, 0f32, 1f32];
    const Y: [f32; 3] = [1f32, 1f32, 0f32];
    const K: [f32; 3] = [0f32, 0f32, 0f32];
    const W: [f32; 3] = [1f32, 1f32, 1f32];
    const GRAY_7: [f32; 3] = [0.7f32, 0.7f32, 0.7f32];

    #[test]
    pub fn test1() {
        let hull = FastHull::new(vec![C, M, Y, K]).unwrap();
        hull.assert_watertight();
        hull.assert_spherical_isoembryonic();
        assert_eq!(
            get_facets(&hull),
            vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]]
        );
    }

    #[test]
    pub fn test2() {
        let mut hull = FastHull::new(vec![C, Y, R, B, GRAY_7]).unwrap();
        hull.assert_watertight();
        hull.assert_spherical_isoembryonic();
        assert_eq!(
            get_facets(&hull),
            vec![
                [0, 1, 3],
                [0, 2, 4],
                [0, 3, 2],
                [0, 4, 1],
                [1, 2, 3],
                [1, 4, 2],
            ]
        );

        log!(":: insert");
        hull.insert(W);
        hull.assert_watertight();
        hull.assert_spherical_isoembryonic();
        assert_eq!(
            get_facets(&hull),
            vec![
                [0, 1, 3],
                [0, 3, 5],
                [0, 5, 1],
                [1, 2, 3],
                [1, 5, 2],
                [2, 5, 3],
            ]
        );
    }

    #[test]
    pub fn test3() {
        let mut hull = FastHull::new(vec![C, R, K, W]).unwrap();
        hull.assert_watertight();
        hull.assert_spherical_isoembryonic();
        assert_eq!(
            get_facets(&hull),
            vec![[0, 1, 3], [0, 2, 1], [0, 3, 2], [1, 2, 3]]
        );

        log!(":: insert");
        hull.insert(B);
        hull.assert_watertight();
        hull.assert_spherical_isoembryonic();
        assert_eq!(
            get_facets(&hull),
            vec![
                [0, 2, 4],
                [0, 3, 2],
                [0, 4, 3],
                [1, 2, 3],
                [1, 3, 4],
                [1, 4, 2],
            ]
        );
    }

    #[test]
    pub fn test4() {
        let hull = FastHull::new(vec![R, G, B]).unwrap();
        hull.assert_watertight();
        hull.assert_spherical_isoembryonic();
        assert_eq!(hull.raw_colors.len(), 3);
        assert_eq!(hull.points.len(), 4);
        for facet in hull.facets.iter() {
            let vs = facet.as_ref().unwrap().vs;
            for v in vs {
                assert!(v < 4);
                assert!(hull.points[v].idx < 3);
            }
        }
    }

    /// 用户报告的22个油漆色案例
    #[test]
    pub fn test_user_case_22() {
        let hex_colors = [
            0xffffff, 0x231f20, 0xed1c24, 0xffdd00, 0x005aaa, 0x006835, 0xca4136, 0xd1d3d4,
            0xdcaf31, 0xcd733a, 0xb4bbbf, 0x6f5f3c, 0x677a83, 0x234f5b, 0x003920, 0x22543f,
            0x51544c, 0x273c4d, 0x9d8145, 0x4dbfa5, 0xaa8a4b, 0x5c5144,
        ];
        let colors: Vec<[f32; 3]> = hex_colors.iter().map(|&h| crate::hex_to_rgb(h)).collect();
        let hull = FastHull::new(colors).unwrap();
        hull.assert_watertight();
    }

    /// 用户报告的29个油漆色案例（含重复色）
    #[test]
    pub fn test_user_case_29() {
        let hex_colors = [
            0xffffff, 0x231f20, 0xed1c24, 0xffdd00, 0x005aaa, 0x006835, 0xca4136, 0xd1d3d4,
            0xdcaf31, 0xcd733a, 0xb4bbbf, 0x6f5f3c, 0x677a83, 0x234f5b, 0x003920, 0x22543f,
            0x51544c, 0x273c4d, 0x9d8145, 0x4dbfa5, 0xaa8a4b, 0x5c5144, 0x4b604d, 0x707863,
            0xcfdaaa, 0xb2a335, 0x231f20, 0x61343b, 0xffffff,
        ];
        let colors: Vec<[f32; 3]> = hex_colors.iter().map(|&h| crate::hex_to_rgb(h)).collect();
        let hull = FastHull::new(colors).unwrap();
        hull.assert_watertight();

        // 检查哪些原始颜色不在凸包表面上（在内部）
        let mut on_hull = vec![false; hex_colors.len()];
        for f in &hull.facets {
            if let Some(f) = f {
                for &v in &f.vs {
                    on_hull[v] = true;
                }
            }
        }
        let interior: Vec<_> = on_hull
            .iter()
            .enumerate()
            .filter(|(_, b)| !**b)
            .map(|(i, _)| i)
            .collect();
        if !interior.is_empty() {
            eprintln!("Interior points (not on hull surface):");
            for &i in &interior {
                let idx = hull.points[i].idx;
                eprintln!(
                    "  point {} -> original color {} (rgb #{:06x})",
                    i,
                    idx,
                    (hex_colors[idx])
                );
            }
        }
        eprintln!(
            "Hull vertices: {}/{}",
            on_hull.iter().filter(|&&b| b).count(),
            hex_colors.len()
        );

        // 验证：所有内部点都确实在所有面的内侧
        for &i in &interior {
            for (fi_idx, f) in hull.facets.iter().enumerate() {
                if let Some(f) = f {
                    if !f.vs.contains(&i) {
                        let side = hull.get_facet_side(&hull.points[i].latent, f.vs);
                        assert!(
                            side <= 0.0,
                            "interior point {} is outside face {} [{} {} {}]",
                            i,
                            fi_idx,
                            f.vs[0],
                            f.vs[1],
                            f.vs[2]
                        );
                    }
                }
            }
        }
    }

    /// 简单确定性伪随机数生成器
    struct XorShift64(u64);
    impl XorShift64 {
        fn new(seed: u64) -> Self {
            let mut s = XorShift64(seed.wrapping_mul(6364136223846793005).wrapping_add(1));
            for _ in 0..5 {
                s.next_u64();
            }
            s
        }
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn next_f32(&mut self) -> f32 {
            (self.next_u64() >> 40) as f32 / (0xFFFFFF as f32)
        }
        fn next_rgb(&mut self) -> [f32; 3] {
            [self.next_f32(), self.next_f32(), self.next_f32()]
        }
    }

    fn assert_valid(hull: &FastHull) {
        hull.assert_watertight();
        hull.assert_spherical_isoembryonic();
    }

    #[test]
    pub fn test_stress_random() {
        for seed in 0..50u64 {
            let mut rng = XorShift64::new(seed);
            let n = 5 + (seed % 16) as usize;
            let colors: Vec<_> = (0..n).map(|_| rng.next_rgb()).collect();
            let mut hull = FastHull::new(colors).unwrap();
            assert_valid(&hull);
            for _ in 0..(2 + (seed % 3) as usize) {
                hull.insert(rng.next_rgb());
                assert_valid(&hull);
            }
        }
    }

    #[test]
    pub fn test_stress_large() {
        for seed in 0..20u64 {
            let mut rng = XorShift64::new(seed);
            let n = 50 + (seed as usize % 51);
            let colors: Vec<_> = (0..n).map(|_| rng.next_rgb()).collect();
            assert_valid(&FastHull::new(colors).unwrap());
        }
    }

    // /// 用射线投射法验证增量算法结果（几何上严格正确）
    // #[test]
    // pub fn test_verify_vs_incremental() {
    //     // 用增量算法建凸包，然后用射线投射法验证
    //     let test_cases: Vec<Vec<[f32; 3]>> = vec![
    //         vec![C, M, Y, K],
    //         vec![C, Y, R, B, GRAY_7, W],
    //         vec![C, R, K, W, B],
    //         vec![R, G, B],
    //     ];
    //     for colors in &test_cases {
    //         let hull = FastHull::new(colors.clone()).unwrap();
    //         // 提取面和顶点
    //         let faces: Vec<verify::Face> = hull
    //             .facets
    //             .iter()
    //             .filter_map(|f| f.as_ref().map(|f| f.vs))
    //             .collect();
    //         let points: Vec<Latent> = hull.points.iter().map(|p| p.latent).collect();
    //         verify::assert_valid(&points, &faces);
    //     }
    // }

    // /// 用户案例：22色 + 射线投射交叉验证
    // #[test]
    // pub fn test_verify_user_22() {
    //     let hex_colors = [
    //         0xffffff, 0x231f20, 0xed1c24, 0xffdd00, 0x005aaa, 0x006835, 0xca4136, 0xd1d3d4,
    //         0xdcaf31, 0xcd733a, 0xb4bbbf, 0x6f5f3c, 0x677a83, 0x234f5b, 0x003920, 0x22543f,
    //         0x51544c, 0x273c4d, 0x9d8145, 0x4dbfa5, 0xaa8a4b, 0x5c5144,
    //     ];
    //     let colors: Vec<_> = hex_colors.iter().map(|&h| crate::hex_to_rgb(h)).collect();
    //     let hull = FastHull::new(colors).unwrap();

    //     // 用 get_facet_side 检查点12对所有面的位置
    //     let p = 12;
    //     eprintln!("Point {} (rgb #677a83):", p);
    //     for (fi_idx, f) in hull.facets.iter().enumerate() {
    //         if let Some(f) = f {
    //             if !f.vs.contains(&p) {
    //                 let side = hull.get_facet_side(&hull.points[p].latent, f.vs);
    //                 if side > 0.0 {
    //                     eprintln!(
    //                         "  OUTSIDE face {} [{},{},{}] side={}",
    //                         fi_idx, f.vs[0], f.vs[1], f.vs[2], side
    //                     );
    //                 }
    //             }
    //         }
    //     }

    //     let faces: Vec<verify::Face> = hull
    //         .facets
    //         .iter()
    //         .filter_map(|f| f.as_ref().map(|f| f.vs))
    //         .collect();
    //     let points: Vec<Latent> = hull.points.iter().map(|p| p.latent).collect();
    //     verify::assert_valid(&points, &faces);
    // }

    #[test]
    pub fn test_user_case_5_plus_8() {
        let initial = [0xffffff, 0x231f20, 0xed1c24, 0xffdd00, 0x005aaa];
        let colors: Vec<_> = initial.iter().map(|&h| crate::hex_to_rgb(h)).collect();
        let mut hull = FastHull::new(colors).unwrap();
        hull.assert_watertight();

        let inserts = [
            0xff0000, 0x00ff00, 0x0000ff, 0x00ffff, 0xff00ff, 0xffff00, 0x000000, 0xffffff,
        ];
        for &h in &inserts {
            hull.insert(crate::hex_to_rgb(h));
            hull.assert_watertight();
            hull.assert_spherical_isoembryonic();
        }

        // 诊断：打印所有面
        eprintln!(
            "=== Final hull ({} faces) ===",
            hull.facets.iter().filter(|x| x.is_some()).count()
        );
        for (i, f) in hull.facets.iter().enumerate() {
            if let Some(f) = f {
                eprintln!("  face {}: [{},{},{}]", i, f.vs[0], f.vs[1], f.vs[2]);
            }
        }
    }
}
