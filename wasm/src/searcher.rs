use std::{cmp::Reverse, collections::HashSet, num::NonZero};

use empfindung::cie00;
use fixedbitset::FixedBitSet;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use lab::Lab;
use mixbox::{float_rgb_to_latent, latent_to_float_rgb};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use smallvec::{SmallVec, smallvec};
use wasm_bindgen::JsError;
use web_sys::console;

type Rgb = [f32; 3];
type Latent = [f32; 7];

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SurfaceType {
    G,
    SG,
    M,
    ME,
    C,
    PA,
    FL,
    W,
    U,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FilterOptions {
    /// 允许的系列（brand+serie）集合；为空表示不限制
    #[serde(default)]
    pub series: Vec<(String, String)>,
    /// 允许的油漆下标（对应 list() 返回的 index）集合；为None表示不限制（不按库存过滤）
    #[serde(default)]
    pub all: Option<Vec<usize>>,

    #[serde(default)]
    pub surfaces: Vec<SurfaceType>,

    #[serde(default)]
    pub bases: Vec<u8>,

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
    /// 每个油漆型号的直接对应关系（例如Gunze H9 <-> Gunze C9），direct_equivs[i]是与majors[i]对应的其他型号下标
    direct_equivs: Vec<Vec<usize>>,
}

#[derive(Debug, Serialize)]
pub struct SearchResultPortion {
    pub t: f32,
    pub brand: String,
    pub code: String,
    pub desc: String,
    pub rgb: Rgb,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    delta_e: f32,
    rgb: Rgb,
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

pub fn color_diff(rgb_a: u32, rgb_b: u32) -> f32 {
    let lab_a = Lab::from_rgb_normalized(&hex_to_rgb(rgb_a));
    let lab_b = Lab::from_rgb_normalized(&hex_to_rgb(rgb_b));
    cie00::diff(&lab_a, &lab_b)
}

fn hex_to_rgb(hex: u32) -> Rgb {
    let b = (hex >> 0) as u8;
    let g = (hex >> 8) as u8;
    let r = (hex >> 16) as u8;
    [(r as f32) / 255.0, (g as f32) / 255.0, (b as f32) / 255.0]
}

fn lerp_latent(l0: &Latent, l1: &Latent, t: f32) -> Latent {
    std::array::from_fn(|i| t * l0[i] + (1f32 - t) * l1[i])
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
    pub fn load(blob: &[u8], equiv_blob: &[u8]) -> Result<Self, JsError> {
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
            let lab = Lab::from_rgb_normalized(&rgb);
            let latent = float_rgb_to_latent(&rgb);

            majors.push(row);
            labs.push(lab);
            latents.push(latent);
        }

        let points: Vec<[f32; 3]> = labs.iter().map(|x| [x.l, x.a, x.b]).collect();
        let kdtree = ImmutableKdTree::new_from_slice(points.as_slice());
        let mut direct_equivs = vec![Vec::new(); majors.len()];

        let equiv_s = std::str::from_utf8(equiv_blob)?;
        let mut equiv_rdr = csv::ReaderBuilder::new()
            .delimiter(b',')
            .has_headers(false)
            .from_reader(equiv_s.as_bytes());
        for v in equiv_rdr.records() {
            let ref rec = v?;
            let a: usize = rec
                .get(0)
                .ok_or_else(|| JsError::new("missing column a"))?
                .parse()?;
            let b: usize = rec
                .get(1)
                .ok_or_else(|| JsError::new("missing column b"))?
                .parse()?;
            if a < direct_equivs.len() && b < direct_equivs.len() {
                direct_equivs[a].push(b);
                direct_equivs[b].push(a);
            }
        }

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

    pub fn search(&self, rgb: u32, opts: &FilterOptions) -> Result<Vec<SearchResult>, JsError> {
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
        let prop_filter: Option<HashSet<SurfaceType>> = if opts.surfaces.is_empty() {
            None
        } else {
            Some(opts.surfaces.iter().map(|x| *x).collect())
        };
        let base_filter: u8 = if opts.bases.is_empty() {
            u8::MAX
        } else {
            opts.bases.iter().fold(0, |acc, e| acc | (1 << e))
        };

        let mut candidates = FixedBitSet::with_capacity(self.majors.len());
        for (i, maj) in self.majors.iter().enumerate() {
            // base
            if (maj.base & base_filter) != 0 {
                // idxs
                if all_filter.as_ref().is_none_or(|s| s.contains(&i)) {
                    // prop
                    if prop_filter.as_ref().is_none_or(|s| s.contains(&maj.prop)) {
                        //serie
                        let serie_ok = series_filter
                            .as_ref()
                            .is_none_or(|s| s.contains(&(maj.brand.as_str(), maj.serie.as_str())));

                        if serie_ok {
                            unsafe {
                                candidates.insert_unchecked(i);
                            }
                        }
                    }
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
            let rgb = latent_to_float_rgb(&latent);
            portions.sort_by_key(|x| Reverse(OrderedFloat(x.t)));
            let portions: Vec<_> = portions
                .into_iter()
                .map(|Portion { t, i }| SearchResultPortion {
                    t: t,
                    brand: self.majors[i].brand.clone(),
                    code: self.majors[i].code.clone(),
                    desc: self.majors[i].desc.clone(),
                    rgb: hex_to_rgb(self.majors[i].rgb),
                })
                .collect();
            results.push(SearchResult {
                delta_e,
                rgb,
                portions,
            });
        }

        // console::log_1(&format!(":: results = {:?}", results).into());

        Ok(results)
    }

    fn search_impl(&self, ctx: &mut SearchContext, rgb: u32, max_mix: u32) -> Vec<SearchMix> {
        let rgb_out = hex_to_rgb(rgb);
        let lab_out = Lab::from_rgb_normalized(&rgb_out);

        let li = self.search_nearest_n(&ctx, &lab_out);
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
                &rgb_out,
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
        rgb_out: &Rgb,
        is: &[usize],
        rem: u32,
        limit: usize,
        base: u8,
    ) -> Vec<SearchMix> {
        let mut results: Vec<SearchMix> = vec![];
        if base == 0 {
            return results;
        }

        // console::log_1(
        //     &format!(
        //         ":: candidates = {:?}",
        //         ctx.candidates.ones().collect::<Vec<_>>()
        //     )
        //     .into(),
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
                    let lab = Lab::from_rgb_normalized(&rgb);
                    self.search_mix_target(ctx, &lab, mix_base)
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
        rgb_out: &Rgb,
        i_0: usize,
        search_next: &dyn Fn(&mut SearchContext, &Rgb) -> Vec<SearchMix>,
    ) -> Vec<SearchMix> {
        let latent_out = float_rgb_to_latent(rgb_out);
        let lab_out = Lab::from_rgb_normalized(rgb_out);
        let latent_0 = &self.latents[i_0];

        let n = ctx.mix2_iter;
        let dt = 0.5f32 / n as f32;

        let mut results: Vec<SearchMix> = vec![];

        for k in 1..=n {
            let t0 = k as f32 * dt;
            let latent_1: Latent =
                std::array::from_fn(|i| (latent_out[i] - t0 * latent_0[i]) / (1f32 - t0));
            let rgb_1 = latent_to_float_rgb(&latent_1);

            // console::log_1(
            //     &format!(
            //         "do_search_mix({}={:?}, dst={:?}) :: {} -> {:?}",
            //         self.majors[i_0].code,
            //         hex_to_rgb(self.majors[i_0].rgb),
            //         rgb_out,
            //         t0,
            //         rgb_1
            //     )
            //     .into(),
            // );
            for SearchMix {
                mut portions,
                mut latent,
                ..
            } in search_next(ctx, &rgb_1)
            {
                // console::log_1(&format!(":: search_next -> {:?}", portions).into());
                let SearchMix2Portion { t, delta_e } =
                    self.search_mix2_portion(ctx, latent_0, &latent, &lab_out, t0, dt * 0.5f32);
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
        lab_out: &Lab,
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

                let rgb = latent_to_float_rgb(&latent);
                let lab = Lab::from_rgb_normalized(&rgb);
                let d = cie00::diff(&lab, &lab_out);

                if d < mind {
                    mink = Some(k);
                    mind = d;
                }
            }

            if let Some(k) = mink {
                t0 = t0 + k as f32 * dt;
            }

            if dt < ctx.mix2_prec {
                // console::log_1(
                //     &format!(
                //         "search_mix2_portion() -> t={} rgb={:?} d={}",
                //         t0,
                //         latent_to_float_rgb(&std::array::from_fn(
                //             |i| t0 * latent_0[i] + (1f32 - t0) * latent_1[i]
                //         )),
                //         mind
                //     )
                //     .into(),
                // );
                return SearchMix2Portion {
                    t: t0,
                    delta_e: mind,
                };
            }
        }
    }

    fn search_nearest_n(&self, ctx: &SearchContext, lab: &Lab) -> Vec<MeasuredItem> {
        let point = [lab.l, lab.a, lab.b];
        let mut results: Vec<_> = if ctx.candidates.count_ones(..) > 3000 {
            // use kdtree
            let nearest_n = self.kdtree.nearest_n::<SquaredEuclidean>(&point, unsafe {
                NonZero::new_unchecked(ctx.limit * 10)
            });
            nearest_n
                .into_iter()
                .filter_map(|x| {
                    let i = x.item as usize;
                    if ctx.candidates.contains(i) {
                        let lab = self.labs[x.item as usize];
                        let d = cie00::diff(point, lab);
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
                    delta_e: cie00::diff(point, self.labs[i]),
                })
                .collect()
        };

        results.sort_by_key(|x| OrderedFloat(x.delta_e));
        results
    }

    fn search_mix_target(&self, ctx: &SearchContext, lab: &Lab, base: u8) -> Option<MeasuredItem> {
        let point = [lab.l, lab.a, lab.b];
        let mut mini = None;
        let mut mind = f32::MAX;
        for i in ctx.candidates.ones() {
            // cannot mix paint of different base
            if (self.majors[i].base & base) != 0 {
                let d = cie00::diff(point, self.labs[i]);
                if d < mind {
                    mini = Some(i);
                    mind = d;
                }
            }
        }

        mini.map(|i| MeasuredItem { i, delta_e: mind })
    }
}
