use core::f32;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    mem::swap,
    sync::{Arc, Mutex},
};

use derivative::Derivative;
use empfindung::ToLab;
use fixedbitset::FixedBitSet;
use glam::Vec3;
use lab::Lab;
use libm::powf;
use mixbox::{float_rgb_to_latent, latent_to_float_rgb};
use once_cell::sync::Lazy;
use ordered_float::OrderedFloat;

use crate::{BoxError, Latent, Rgb, lerp_latent, log};

static TESSELLATOR: Lazy<Mutex<Tessellator>> = Lazy::new(|| Mutex::new(Tessellator::new()));

#[inline(always)]
fn srgb_to_linear(x: f32) -> f32 {
    if x >= 0.04045 {
        powf((x + 0.055) / 1.055, 2.4)
    } else {
        x / 12.92
    }
}

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
}

// #[derive(Debug)]
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

#[derive(Debug)]
pub struct HullMesh {
    pub positions: Vec<f32>,
    pub colors: Vec<f32>,
    pub indices: Vec<u32>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Hull {
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

    #[derivative(Debug = "ignore")]
    mesh: Option<HullMesh>,
}

#[derive(Clone)]
struct Tessellation {
    uvs: Arc<Vec<(f32, f32)>>,
    triangles: Arc<Vec<usize>>,
}

struct Tessellator {
    lookup: HashMap<[usize; 3], Tessellation>,
}

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "({},{},{})",
            self.lab.l, self.lab.a, self.lab.b,
        ))
    }
}

impl Display for Facet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<{},{},{}>",
            self.vs[0], self.vs[1], self.vs[2],
        ))
    }
}

impl Debug for Facet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<{},{},{}>::[{},{},{}]",
            self.vs[0], self.vs[1], self.vs[2], self.fs[0], self.fs[1], self.fs[2],
        ))
    }
}

impl Display for Hull {
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

impl Tessellator {
    fn new() -> Self {
        Tessellator {
            lookup: HashMap::new(),
        }
    }

    fn get(&mut self, key: &[usize; 3]) -> Tessellation {
        // 把尺寸比较小的细分结果缓存起来，避免频繁做三角剖分
        if key.iter().all(|x| *x <= 3) {
            if let Some(result) = self.lookup.get(key) {
                return result.clone();
            }
            let result = Self::compute_subdivision(key);
            self.lookup.insert(*key, result.clone());
            return result;
        }

        Self::compute_subdivision(key)
    }

    fn compute_subdivision(key: &[usize; 3]) -> Tessellation {
        let mut uvs = vec![
            delaunator::Point { x: 0.0, y: 0.0 },
            delaunator::Point { x: 1.0, y: 0.0 },
            delaunator::Point { x: 0.0, y: 1.0 },
        ];
        let [a, b, c] = *key;

        for i in 1..c {
            let t = i as f64 / c as f64;
            uvs.push(delaunator::Point { x: t, y: 0.0 });
        }

        for i in 1..b {
            let t = i as f64 / b as f64;
            uvs.push(delaunator::Point { x: 0.0, y: t });
        }

        for i in 1..a {
            let t = i as f64 / a as f64;
            uvs.push(delaunator::Point { x: t, y: 1.0 - t });
        }

        let n_in = (a + b + c) / 3 - 1;
        if n_in > 1 {
            for i in 1..n_in {
                for j in 1..=(n_in - i) {
                    let u = i as f64 / n_in as f64;
                    let v = j as f64 / n_in as f64;
                    uvs.push(delaunator::Point { x: u, y: v });
                }
            }
        }

        // log!(":: subdivide => {:?} \n{:?}", key, uvs);
        let dt = delaunator::triangulate(&uvs);
        let uvs: Vec<_> = uvs.iter().map(|p| (p.x as f32, p.y as f32)).collect();
        // log!(":: key = {:?} uvs = {}", key, uvs.len());
        // log!(":: triangles = {:?}", dt.triangles);

        Tessellation {
            uvs: Arc::new(uvs),
            triangles: Arc::new(dt.triangles),
        }
    }
}

impl Hull {
    pub fn new(raw_colors: Vec<Rgb>) -> Result<Hull, BoxError> {
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
            let lab = sample_microsphere(&points[idx].lab, 1e-5);
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

        let mut hull = Hull {
            raw_colors,
            points,
            facets: vec![],
            null_facets: vec![],
            center_latent,
            center,
            mesh: None,
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

        log!(":: new :: {}", hull);

        Ok(hull)
    }

    pub fn insert(&mut self, color: Rgb) {
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

    pub fn mesh(&self) -> &HullMesh {
        self.mesh.as_ref().unwrap()
    }

    fn build_mesh(&mut self) {
        let mut positions = vec![];
        let mut colors = vec![];
        let mut indices = vec![];

        let mut i0 = 0;
        for facet in self.facets.iter() {
            if let Some(facet) = facet.as_ref() {
                let (vertices, triangles) = self.build_facet_mesh(facet.vs);

                let vertices = vertices.as_slice();
                let triangles = triangles.as_slice();

                for xs in vertices.iter() {
                    let [l, a, b] = xs.to_array();
                    positions.push(l);
                    positions.push(a);
                    positions.push(b);
                    let lab = Lab { l, a, b };
                    let srgb = lab.to_rgb_normalized();
                    let [r, g, b] = srgb.map(srgb_to_linear);
                    colors.push(r);
                    colors.push(g);
                    colors.push(b);
                }

                for i in triangles.iter() {
                    indices.push(i0 + *i as u32);
                }

                i0 += vertices.len() as u32;
            }
        }

        self.mesh = Some(HullMesh {
            positions,
            colors,
            indices,
        });
    }

    fn build_facet_mesh(&self, vs: [usize; 3]) -> (Vec<Vec3>, Arc<Vec<usize>>) {
        let labs = vs.map(|v| &self.points[v].lab);
        let latents = vs.map(|v| &self.points[v].latent);

        let subdivisions = [
            get_subdivision_count(&labs[1], &labs[2]),
            get_subdivision_count(&labs[0], &labs[2]),
            get_subdivision_count(&labs[0], &labs[1]),
        ];

        let mut subdivision = TESSELLATOR.lock().unwrap();
        let result = subdivision.get(&subdivisions);

        let vertices = result
            .uvs
            .iter()
            .map(|(t1, t2)| {
                let latent: Latent = std::array::from_fn(|i| {
                    t1 * latents[1][i] + t2 * latents[2][i] + (1.0 - t1 - t2) * latents[0][i]
                });
                let rgb = latent_to_float_rgb(&latent);
                let (l, a, b) = Lab::from_rgb_normalized(&rgb).to_lab();
                Vec3::new(l, a, b)
            })
            .collect();

        (vertices, result.triangles.clone())
    }

    // 检测点p在ccw曲面vs的哪一侧
    fn get_facet_side(&self, p: &Latent, vs: [usize; 3]) -> f32 {
        let t = 0.01;
        let [a, b, c] = vs;

        // 求曲面在a点处的切平面a-db-dc，以及曲线ap在a点处的切线a-dp。根据物理意义得知，ap和abc不会有除a以外的交点。
        // 因此若dp在a-db-dc的+侧，p一定在abc的+侧，反之亦然。
        let latent_a = &self.points[a].latent;
        let latent_b = lerp_latent(&self.points[b].latent, latent_a, t);
        let latent_c = lerp_latent(&self.points[c].latent, latent_a, t);
        let latent_p = lerp_latent(p, latent_a, t);

        let va = lab_to_vec3(&self.points[a].lab);
        let vb = lab_to_vec3(&latent_to_lab(&latent_b));
        let vc = lab_to_vec3(&latent_to_lab(&latent_c));
        let vp = lab_to_vec3(&latent_to_lab(&latent_p));

        let vab = vb - va;
        let vac = vc - va;
        let vap = vp - va;

        let n = vab.cross(vac).normalize();
        let vap = vap.normalize();

        // log!("  :: n={} vap={}", n, vap);
        log!("  :: get_facet_side <{},{},{}> = {}", a, b, c, n.dot(vap));

        n.dot(vap)
    }

    fn try_fold_facet(&mut self, i: usize) -> bool {
        let fs = self.facets[i].as_ref().unwrap().fs;

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

        log!("{}", self);

        for i in [i0, i1, i2] {
            self.remesh_facet_adj_flood(i, 0, &mut visited);
        }
    }

    fn remesh_facet_adj_flood(&mut self, i: usize, vi_idx: usize, visited: &mut FixedBitSet) {
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
            ":: in tetrahedron :: {} {} {} {} :: m = {}",
            vi,
            va,
            vb,
            vj,
            lab_to_vec3(&latent_to_lab(&latent_c))
        );

        // 重心在fi的外面，说明fi失效了，显然fj也失效了，需要做remesh。
        let s1 = self.get_facet_side(&latent_c, fi.vs);
        let s2 = self.get_facet_side(&latent_c, fj.vs);

        // 有些四面体可能非常扁平，在这种位置求梯度可能误差很大，取两个里面更可信的一个
        let side = if s1.abs() > s2.abs() { s1 } else { s2 };

        if side > 0.0 {
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

            log!("{}", self);

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
mod tests {
    use crate::{
        log,
        sim::{Facet, Hull},
    };

    fn get_facets(hull: &Hull) -> Vec<[usize; 3]> {
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

    fn assert_watertight(hull: &Hull) {
        for (i, fi) in hull.facets.iter().enumerate() {
            if let Some(fi) = fi {
                for (vi, fj) in fi.fs.iter().enumerate() {
                    let fj = hull.facets[*fj].as_ref().unwrap();
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
        let hull = Hull::new(vec![C, M, Y, K]).unwrap();
        assert_watertight(&hull);
        assert_eq!(
            get_facets(&hull),
            vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]]
        );
    }

    #[test]
    pub fn test2() {
        let mut hull = Hull::new(vec![C, Y, R, B, GRAY_7]).unwrap();
        assert_watertight(&hull);
        assert_eq!(
            get_facets(&hull),
            vec![
                [0, 1, 3],
                [0, 3, 4],
                [0, 4, 1],
                [1, 2, 3],
                [1, 4, 2],
                [2, 4, 3],
            ]
        );

        log!(":: insert");
        hull.insert(W);
        assert_watertight(&hull);
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
        let mut hull = Hull::new(vec![C, R, K, W]).unwrap();
        assert_watertight(&hull);
        assert_eq!(
            get_facets(&hull),
            vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]]
        );

        log!(":: insert");
        hull.insert(B);
        assert_watertight(&hull);
        assert_eq!(
            get_facets(&hull),
            vec![
                [0, 1, 2],
                [0, 2, 4],
                [0, 3, 1],
                [0, 4, 3],
                [1, 3, 4],
                [1, 4, 2],
            ]
        );
    }

    #[test]
    pub fn test4() {
        let hull = Hull::new(vec![R, G, B]).unwrap();
        assert_watertight(&hull);
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
}
