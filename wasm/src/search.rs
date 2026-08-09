use std::{cmp::Reverse, collections::HashSet, fmt, num::NonZero};

use bitflags::bitflags;
use fixedbitset::FixedBitSet;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use smallvec::{SmallVec, smallvec};

use crate::{
    BoxError, Latent, Oklab, Rgb, hex_to_rgb, latent_to_rgb, lerp_latent, oklab_dist,
    rgb_to_latent, rgb_to_oklab,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaintInfo {
    /// 在majors/list()里的下标；从csv按行位置解析时是跳过的（不占列），load()里按顺序补上
    #[serde(skip_deserializing, default)]
    pub index: usize,
    pub brand: String,
    pub serie: String,
    pub code: String,
    pub rgb: u32,
    pub desc: String,
    pub base: u8,
    pub prop: SurfaceType,
}

bitflags! {
    /// 漆面类型，每个变体占一个 bit；`prop` 字段是单 bit 值，过滤时是多个 bit 的 mask
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SurfaceType: u16 {
        const G = 1 << 0;
        const SG = 1 << 1;
        const M = 1 << 2;
        const ME = 1 << 3;
        const C = 1 << 4;
        const PA = 1 << 5;
        const FL = 1 << 6;
        const W = 1 << 7;
        const U = 1 << 8;
    }
}

impl Default for SurfaceType {
    fn default() -> Self {
        SurfaceType::empty()
    }
}

/// 序列化输出整数（bit值/mask，紧凑、快）；反序列化兼容：
/// - 字符串名（"G"、"SG"…，csv / 旧数据）
/// - 整数（单bit值或mask，JS 传入）
/// - 数组（旧 Web 端传入的 ["G","SG"] 或 [1,2]），自动 OR 成 mask
impl Serialize for SurfaceType {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u16(self.bits())
    }
}

impl<'de> Deserialize<'de> for SurfaceType {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = SurfaceType;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "a surface type name, bitmask integer, or an array of them"
                )
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                SurfaceType::from_name(v)
                    .ok_or_else(|| E::custom(format!("unknown surface type: {v}")))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(SurfaceType::from_bits_retain(v as u16))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(SurfaceType::from_bits_retain(v as u16))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
                if v.fract() == 0.0 {
                    Ok(SurfaceType::from_bits_retain(v as u16))
                } else {
                    Err(E::custom(format!("non-integer surface bitmask: {v}")))
                }
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let mut acc = 0u16;
                while let Some(x) = seq.next_element::<SurfaceType>()? {
                    acc |= x.bits();
                }
                Ok(SurfaceType::from_bits_retain(acc))
            }
        }
        d.deserialize_any(V)
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FilterOptions {
    /// 允许的系列（brand+serie）集合；为空表示不限制
    #[serde(default)]
    pub series: Vec<(String, String)>,
    /// 允许的油漆下标（对应 list() 返回的 index）集合；为None表示不限制（不按库存过滤）
    #[serde(default)]
    pub all: Option<Vec<usize>>,

    /// 允许的漆面类型 bitmask；NONE（默认）表示不限制。
    /// 反序列化兼容旧格式字符串/数字数组（如 ["G","SG"]）和新格式整数 mask
    #[serde(default)]
    pub surfaces: SurfaceType,

    #[serde(default)]
    pub bases: Vec<u8>,

    #[serde(default)]
    pub mix: u32,

    #[serde(default)]
    pub limit: Option<usize>,
}

pub struct Searcher {
    majors: Vec<PaintInfo>,
    labs: Vec<Oklab>,
    latents: Vec<Latent>,
    kdtree: ImmutableKdTree<f32, 3>,
    /// 每个油漆型号的直接对应关系（例如Gunze H9 <-> Gunze C9），direct_equivs[i]是与majors[i]对应的其他型号下标
    direct_equivs: Vec<Vec<usize>>,
}

#[derive(Debug, Serialize)]
pub struct SearchResultPortion {
    pub t: f32,
    pub brand: String,
    pub code: String,
    pub desc: String,
    pub rgb: [f32; 3],
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    delta_e: f32,
    rgb: [f32; 3],
    portions: Vec<SearchResultPortion>,
}

struct SearchContext {
    candidates: FixedBitSet,
    limit: usize,
    mix_limit: usize,
    mix2_prec: f32,
    mix2_iter: usize,
}

#[derive(Debug)]
struct Portion {
    t: f32,
    i: usize,
}

#[derive(Debug)]
struct SearchMix {
    portions: SmallVec<[Portion; 8]>,
    latent: Latent,
    delta_e: f32,
}

#[derive(Debug)]
struct SearchMix2Portion {
    t: f32,
    delta_e: f32,
}

#[derive(Debug)]
struct MeasuredItem {
    i: usize,
    delta_e: f32,
}

fn collect_mix_dedup(results: &mut Vec<SearchMix>, new: impl Iterator<Item = SearchMix>) {
    for mix in new {
        if let Some(x) = results.iter_mut().find(|x| {
            x.portions
                .iter()
                .zip(mix.portions.iter())
                .all(|(a, b)| a.i == b.i)
        }) {
            if mix.delta_e < x.delta_e {
                *x = mix;
            }
        } else {
            results.push(mix);
        }
    }
}

impl Searcher {
    /// blob是colors.csv；equiv_blob是一份"a,b"两列下标的csv（下标对应majors/list()的index），
    /// 代表两个型号互为直接等价（例如Gunze H9 <-> Gunze C9）
    pub fn load(blob: &[u8], equiv_blob: &[u8]) -> Result<Self, BoxError> {
        let s = std::str::from_utf8(blob)?;

        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b',')
            .from_reader(s.as_bytes());

        let mut majors = vec![];
        let mut labs = vec![];
        let mut latents = vec![];

        for v in rdr.records() {
            let ref rec = v?;
            let mut row: PaintInfo = rec.deserialize(None)?;
            row.index = majors.len();

            let rgb = hex_to_rgb(row.rgb);
            let lab = rgb_to_oklab(rgb);
            let latent = rgb_to_latent(rgb);

            majors.push(row);
            labs.push(lab);
            latents.push(latent);
        }

        let points: Vec<[f32; 3]> = labs.iter().map(|x| [x.l, x.a, x.b]).collect();
        let kdtree = ImmutableKdTree::new_from_slice(points.as_slice());

        let direct_equivs = vec![Vec::new(); majors.len()];
        // let mut direct_equivs = vec![Vec::new(); majors.len()];
        // let equiv_s = std::str::from_utf8(equiv_blob)?;
        // let mut equiv_rdr = csv::ReaderBuilder::new()
        //     .delimiter(b',')
        //     .has_headers(false)
        //     .from_reader(equiv_s.as_bytes());
        // for v in equiv_rdr.records() {
        //     let ref rec = v?;
        //     let a: usize = rec
        //         .get(0)
        //         .ok_or_else(|| BoxError::new("missing column a"))?
        //         .parse()?;
        //     let b: usize = rec
        //         .get(1)
        //         .ok_or_else(|| BoxError::new("missing column b"))?
        //         .parse()?;
        //     if a < direct_equivs.len() && b < direct_equivs.len() {
        //         direct_equivs[a].push(b);
        //         direct_equivs[b].push(a);
        //     }
        // }

        Ok(Searcher {
            majors,
            labs,
            latents,
            kdtree,
            direct_equivs,
        })
    }

    pub fn direct_equivalences(&self, index: usize) -> Vec<&PaintInfo> {
        let Some(indices) = self.direct_equivs.get(index) else {
            return vec![];
        };
        indices.iter().map(|&i| &self.majors[i]).collect()
    }

    pub fn list(&self) -> Vec<&PaintInfo> {
        self.majors.iter().collect()
    }

    pub fn search(&self, rgb: u32, opts: &FilterOptions) -> Result<Vec<SearchResult>, BoxError> {
        let series_filter: Option<HashSet<(&str, &str)>> = if opts.series.is_empty() {
            None
        } else {
            Some(
                opts.series
                    .iter()
                    .map(|(a, b)| (a.as_str(), b.as_str()))
                    .collect(),
            )
        };
        let all_filter: Option<HashSet<usize>> =
            opts.all.as_ref().map(|ids| ids.iter().copied().collect());
        let prop_mask = opts.surfaces;
        let base_filter: u8 = if opts.bases.is_empty() {
            u8::MAX
        } else {
            opts.bases.iter().fold(0, |acc, e| acc | (1 << e))
        };

        let mut candidates = FixedBitSet::with_capacity(self.majors.len());
        for (i, maj) in self.majors.iter().enumerate() {
            // base + idxs + prop + serie
            if (maj.base & base_filter) != 0
                && all_filter.as_ref().is_none_or(|s| s.contains(&i))
                && (prop_mask.is_empty() || prop_mask.intersects(maj.prop))
                && series_filter
                    .as_ref()
                    .is_none_or(|s| s.contains(&(maj.brand.as_str(), maj.serie.as_str())))
            {
                unsafe {
                    candidates.insert_unchecked(i);
                }
            }
        }

        let limit = opts.limit.unwrap_or(10);
        let mut ctx = SearchContext {
            candidates,
            limit,
            mix_limit: limit,
            mix2_prec: 0.01,
            mix2_iter: 5,
        };

        let mut results = vec![];
        let max_mix = if let Some(_) = opts.all { opts.mix } else { 0 };

        for SearchMix {
            mut portions,
            latent,
            delta_e,
        } in self.search_impl(&mut ctx, rgb, max_mix)
        {
            let rgb = latent_to_rgb(&latent);
            portions.sort_by_key(|x| Reverse(OrderedFloat(x.t)));
            let portions: Vec<_> = portions
                .into_iter()
                .map(|Portion { t, i }| {
                    let major = self.majors.get(i).unwrap();
                    let rgb = hex_to_rgb(self.majors[i].rgb);
                    SearchResultPortion {
                        t: t,
                        brand: major.brand.clone(),
                        code: major.code.clone(),
                        desc: major.desc.clone(),
                        rgb: [rgb.r, rgb.g, rgb.b],
                    }
                })
                .collect();
            results.push(SearchResult {
                delta_e,
                rgb: [rgb.r, rgb.g, rgb.b],
                portions,
            });
        }

        // log!(":: results = {:?}", results);

        Ok(results)
    }

    fn search_impl(&self, ctx: &mut SearchContext, rgb: u32, max_mix: u32) -> Vec<SearchMix> {
        let rgb_out = hex_to_rgb(rgb);
        let lab_out = rgb_to_oklab(rgb_out);

        let li = self.search_nearest_n(&ctx, lab_out);
        let mut results: Vec<_> = li
            .iter()
            .map(|x| SearchMix {
                delta_e: x.delta_e,
                latent: self.latents[x.i],
                portions: smallvec![Portion { t: 1f32, i: x.i }],
            })
            .collect();

        // TODO: use this or iter candidates
        let li1: Vec<_> = li.iter().map(|x| x.i).collect();

        for rem in 1..=max_mix {
            results.append(&mut self.search_mix(
                ctx,
                rgb_out,
                li1.as_slice(),
                rem,
                ctx.limit * 3,
                u8::MAX,
            ));
        }

        results.sort_by_key(|x| OrderedFloat(x.delta_e));
        results.into_iter().take(ctx.limit).collect()
    }

    fn search_mix(
        &self,
        ctx: &mut SearchContext,
        rgb_out: Rgb<f32>,
        is: &[usize],
        rem: u32,
        limit: usize,
        base: u8,
    ) -> Vec<SearchMix> {
        let mut results: Vec<SearchMix> = vec![];
        if base == 0 {
            return results;
        }

        // log!(
        //     ":: candidates = {:?}",
        //     ctx.candidates.ones().collect::<Vec<_>>()
        // );

        for i in is.iter() {
            ctx.candidates.set(*i, false);
            let mix_base = base & self.majors[*i].base;
            if mix_base == 0 {
                continue;
            }

            let li = match rem {
                0 => vec![],
                1 => self.do_search_mix(ctx, rgb_out, *i, &|ctx, rgb| {
                    let lab = rgb_to_oklab(rgb);
                    self.search_mix_target(ctx, lab, mix_base)
                        .map(|MeasuredItem { i, delta_e }| SearchMix {
                            portions: smallvec![Portion { i, t: 1f32 }],
                            latent: self.latents[i],
                            delta_e,
                        })
                        .into_iter()
                        .collect()
                }),
                _ => self.do_search_mix(ctx, rgb_out, *i, &|ctx, rgb| {
                    let li: Vec<_> = ctx
                        .candidates
                        .ones()
                        .filter(|i| (self.majors[*i].base & mix_base) != 0)
                        .collect();
                    self.search_mix(
                        ctx,
                        rgb,
                        li.as_slice(),
                        rem - 1,
                        limit * 3 / is.len(),
                        mix_base,
                    )
                }),
            };

            collect_mix_dedup(&mut results, li.into_iter());

            ctx.candidates.set(*i, true);
        }

        results.sort_by_key(|x| OrderedFloat(x.delta_e));
        results.into_iter().take(limit).collect()
    }

    fn do_search_mix(
        &self,
        ctx: &mut SearchContext,
        rgb_out: Rgb<f32>,
        i_0: usize,
        search_next: &dyn Fn(&mut SearchContext, Rgb<f32>) -> Vec<SearchMix>,
    ) -> Vec<SearchMix> {
        let latent_out = rgb_to_latent(rgb_out);
        let lab_out = rgb_to_oklab(rgb_out);
        let latent_0 = &self.latents[i_0];

        let n = ctx.mix2_iter;
        let dt = 0.5f32 / n as f32;

        let mut results: Vec<SearchMix> = vec![];

        for k in 1..=n {
            let t0 = k as f32 * dt;
            let latent_1: Latent =
                std::array::from_fn(|i| (latent_out[i] - t0 * latent_0[i]) / (1f32 - t0));
            let rgb_1 = latent_to_rgb(&latent_1);

            // log!(
            //     "do_search_mix({}={:?}, dst={:?}) :: {} -> {:?}",
            //     self.majors[i_0].code,
            //     hex_to_rgb(self.majors[i_0].rgb),
            //     rgb_out,
            //     t0,
            //     rgb_1
            // );
            for SearchMix {
                mut portions,
                mut latent,
                ..
            } in search_next(ctx, rgb_1)
            {
                // log!(":: search_next -> {:?}", portions);
                let SearchMix2Portion { t, delta_e } =
                    self.search_mix2_portion(ctx, latent_0, &latent, lab_out, t0, dt * 0.5f32);
                for e in portions.iter_mut() {
                    e.t *= 1f32 - t;
                }
                portions.push(Portion { t, i: i_0 });
                latent = lerp_latent(&latent_0, &latent, t);
                collect_mix_dedup(
                    &mut results,
                    std::iter::once(SearchMix {
                        portions,
                        latent,
                        delta_e,
                    }),
                );
            }
        }

        for x in results.iter_mut() {
            x.portions.sort_by_key(|x| x.i);
        }

        results.sort_by_key(|x| OrderedFloat(x.delta_e));
        results
    }

    fn search_mix2_portion(
        &self,
        ctx: &SearchContext,
        latent_0: &Latent,
        latent_1: &Latent,
        lab_out: Oklab,
        mut t0: f32,
        mut dt: f32,
    ) -> SearchMix2Portion {
        let n = ctx.mix2_iter as i32;

        let mut mind: f32 = f32::MAX;
        loop {
            dt /= n as f32;

            let mut mink = None;
            for k in -n..=n {
                let t = t0 + k as f32 * dt;
                let latent = std::array::from_fn(|i| t * latent_0[i] + (1f32 - t) * latent_1[i]);

                let rgb = latent_to_rgb(&latent);
                let lab = rgb_to_oklab(rgb);
                let d = oklab_dist(lab, lab_out);

                if d < mind {
                    mink = Some(k);
                    mind = d;
                }
            }

            if let Some(k) = mink {
                t0 = t0 + k as f32 * dt;
            }

            if dt < ctx.mix2_prec {
                // log!(
                //     "search_mix2_portion() -> t={} rgb={:?} d={}",
                //     t0,
                //     latent_to_float_rgb(&std::array::from_fn(
                //         |i| t0 * latent_0[i] + (1f32 - t0) * latent_1[i]
                //     )),
                //     mind
                // );
                return SearchMix2Portion {
                    t: t0,
                    delta_e: mind,
                };
            }
        }
    }

    fn search_nearest_n(&self, ctx: &SearchContext, lab: Oklab) -> Vec<MeasuredItem> {
        let mut results: Vec<_> = if ctx.candidates.count_ones(..) > 3000 {
            // use kdtree
            let nearest_n = self
                .kdtree
                .nearest_n::<SquaredEuclidean>(&[lab.l, lab.a, lab.b], unsafe {
                    NonZero::new_unchecked(ctx.limit * 10)
                });
            nearest_n
                .into_iter()
                .filter_map(|x| {
                    let i = x.item as usize;
                    if ctx.candidates.contains(i) {
                        let d = oklab_dist(lab, self.labs[x.item as usize]);
                        Some(MeasuredItem { i, delta_e: d })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            ctx.candidates
                .ones()
                .map(|i| MeasuredItem {
                    i,
                    delta_e: oklab_dist(lab, self.labs[i]),
                })
                .collect()
        };

        results.sort_by_key(|x| OrderedFloat(x.delta_e));
        results
    }

    fn search_mix_target(&self, ctx: &SearchContext, lab: Oklab, base: u8) -> Option<MeasuredItem> {
        let mut mini = None;
        let mut mind = f32::MAX;
        for i in ctx.candidates.ones() {
            // cannot mix paint of different base
            if (self.majors[i].base & base) != 0 {
                let d = oklab_dist(lab, self.labs[i]);
                if d < mind {
                    mini = Some(i);
                    mind = d;
                }
            }
        }

        mini.map(|i| MeasuredItem { i, delta_e: mind })
    }
}
