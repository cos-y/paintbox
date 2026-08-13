use std::{cmp::Reverse, collections::HashSet, fmt, marker::PhantomData, num::NonZero};

use bitflags::bitflags;
use empfindung::cie00;
use fixedbitset::FixedBitSet;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use lab::Lab;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use smallvec::{SmallVec, smallvec};

use crate::{
    BoxError, Latent, Rgb, hex_to_rgb, latent_to_rgb, lerp_latent, rgb_to_lab, rgb_to_latent,
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
    pub bases: BaseType,
    pub surfaces: SurfaceType,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BaseType: u16 {
        const Laquer = 1 << 0;
        const Alcohol = 1 << 1;
        const Enamel = 1 << 2;
        const Water = 1 << 3;
    }

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

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MediaType: u16 {
        const Airbrush = 1 << 0;
        const Spray = 1 << 1;
        const Brush = 1 << 2;
        const Marker = 1 << 3;
        const Other = 1 << 7;
    }
}

fn bitflags_coerce<T>(e: T) -> T
where
    T: bitflags::Flags<Bits = u16>,
{
    if e.is_empty() { T::all() } else { e }
}

/// 序列化为 u16 bitmask（两类型共用）
fn serialize_flags<S, F>(flags: &F, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    F: bitflags::Flags<Bits = u16>,
{
    s.serialize_u16(flags.bits())
}

/// 反序列化：兼容字符串名 / 整数 / 数组（两类型共用）
fn deserialize_flags<'de, D, F>(d: D) -> Result<F, D::Error>
where
    D: serde::Deserializer<'de>,
    F: bitflags::Flags<Bits = u16> + serde::de::Deserialize<'de>,
{
    struct V<F>(PhantomData<F>);

    impl<'de, F: bitflags::Flags<Bits = u16> + serde::de::Deserialize<'de>> serde::de::Visitor<'de>
        for V<F>
    {
        type Value = F;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "a flag type name, bitmask integer, or an array of them")
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            F::from_name(v).ok_or_else(|| E::custom(format!("unknown flag type: {v}")))
        }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(F::from_bits_retain(v as u16))
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(F::from_bits_retain(v as u16))
        }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
            if v.fract() == 0.0 {
                Ok(F::from_bits_retain(v as u16))
            } else {
                Err(E::custom(format!("non-integer bitmask: {v}")))
            }
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let mut acc = 0u16;
            while let Some(x) = seq.next_element::<F>()? {
                acc |= x.bits();
            }
            Ok(F::from_bits_retain(acc))
        }
    }
    d.deserialize_any(V(PhantomData))
}

impl Serialize for BaseType {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        serialize_flags(self, s)
    }
}

impl<'de> Deserialize<'de> for BaseType {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        deserialize_flags(d)
    }
}

impl Default for BaseType {
    fn default() -> Self {
        BaseType::all()
    }
}

impl Serialize for SurfaceType {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        serialize_flags(self, s)
    }
}

impl<'de> Deserialize<'de> for SurfaceType {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        deserialize_flags(d)
    }
}

impl Default for SurfaceType {
    fn default() -> Self {
        SurfaceType::all()
    }
}

impl Serialize for MediaType {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        serialize_flags(self, s)
    }
}

impl<'de> Deserialize<'de> for MediaType {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        deserialize_flags(d)
    }
}

impl Default for MediaType {
    fn default() -> Self {
        MediaType::all()
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
    pub medias: MediaType,

    #[serde(default)]
    pub bases: BaseType,

    #[serde(default)]
    pub mix: u32,

    #[serde(default)]
    pub limit: Option<usize>,
}

pub struct Searcher {
    majors: Vec<PaintInfo>,
    labs: Vec<Lab>,
    latents: Vec<Latent>,
    kdtree: ImmutableKdTree<f32, 3>,
}

#[derive(Debug, Serialize)]
pub struct SearchResultPortion {
    pub t: f32,
    pub brand: String,
    pub code: String,
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
            let lab = rgb_to_lab(rgb);
            let latent = rgb_to_latent(rgb);

            majors.push(row);
            labs.push(lab);
            latents.push(latent);
        }

        let points: Vec<[f32; 3]> = labs.iter().map(|x| [x.l, x.a, x.b]).collect();
        let kdtree = ImmutableKdTree::new_from_slice(points.as_slice());

        Ok(Searcher {
            majors,
            labs,
            latents,
            kdtree,
        })
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

        let surfaces = bitflags_coerce(opts.surfaces);
        let bases = bitflags_coerce(opts.bases);
        let medias = bitflags_coerce(opts.medias);

        let mut candidates = FixedBitSet::with_capacity(self.majors.len());
        for (i, maj) in self.majors.iter().enumerate() {
            // base + idxs + prop + serie
            if bases.intersects(maj.bases)
                && surfaces.intersects(maj.surfaces)
                && all_filter.as_ref().is_none_or(|s| s.contains(&i))
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
        let lab_out = rgb_to_lab(rgb_out);

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
            // 这里无需限制bases，因为只会从candidates里选择，而candidates已经在前面过滤过了
            results.append(&mut self.search_mix(
                ctx,
                rgb_out,
                li1.as_slice(),
                rem,
                ctx.limit * 3,
                BaseType::all(),
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
        bases: BaseType,
    ) -> Vec<SearchMix> {
        // base限制当前搜索分支允许的溶剂种类，为空表示可剪枝
        let mut results: Vec<SearchMix> = vec![];
        if bases.is_empty() {
            return results;
        }

        // log!(
        //     ":: candidates = {:?}",
        //     ctx.candidates.ones().collect::<Vec<_>>()
        // );

        for i in is.iter() {
            ctx.candidates.set(*i, false);
            let mix_bases = bases.intersection(self.majors[*i].bases);
            if mix_bases.is_empty() {
                continue;
            }

            let li = match rem {
                0 => vec![],
                1 => self.do_search_mix(ctx, rgb_out, *i, &|ctx, rgb| {
                    let lab = rgb_to_lab(rgb);
                    self.search_mix_target(ctx, lab, mix_bases)
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
                        .filter(|i| mix_bases.intersects(self.majors[*i].bases))
                        .collect();
                    self.search_mix(
                        ctx,
                        rgb,
                        li.as_slice(),
                        rem - 1,
                        limit * 3 / is.len(),
                        mix_bases,
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
        let lab_out = rgb_to_lab(rgb_out);
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
        lab_out: Lab,
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
                let lab = rgb_to_lab(rgb);
                let d = cie00::diff(lab, lab_out);

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

    fn search_nearest_n(&self, ctx: &SearchContext, lab: Lab) -> Vec<MeasuredItem> {
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
                        let d = cie00::diff(lab, self.labs[x.item as usize]);
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
                    delta_e: cie00::diff(lab, self.labs[i]),
                })
                .collect()
        };

        results.sort_by_key(|x| OrderedFloat(x.delta_e));
        results
    }

    fn search_mix_target(
        &self,
        ctx: &SearchContext,
        lab: Lab,
        bases: BaseType,
    ) -> Option<MeasuredItem> {
        let mut mini = None;
        let mut mind = f32::MAX;
        for i in ctx.candidates.ones() {
            // cannot mix paint of different base
            if bases.intersects(self.majors[i].bases) {
                let d = cie00::diff(lab, self.labs[i]);
                if d < mind {
                    mini = Some(i);
                    mind = d;
                }
            }
        }

        mini.map(|i| MeasuredItem { i, delta_e: mind })
    }
}
