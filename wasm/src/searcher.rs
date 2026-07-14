use std::{cmp::Reverse, num::NonZero};

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
    pub brand: String,
    pub code: String,
    pub desc: String,
    pub serie: String,
    pub serie_code: String,
    pub rgb: u32,
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
    kdtree_nearest_max_qty: NonZero<usize>,
    mix_limit: usize,
    mix2_prec: f32,
    mix2_iter: usize,
}

struct Portion {
    t: f32,
    i: usize,
}

struct SearchMix {
    portions: SmallVec<[Portion; 8]>,
    latent: Latent,
    delta_e: f32,
}

struct SearchMix2Portion {
    t: f32,
    delta_e: f32,
}

struct SearchNearest {
    i: usize,
    delta_e: f32,
}

fn hex_to_rgb(hex: u32) -> Rgb {
    let b = (hex >> 0) as u8;
    let g = (hex >> 8) as u8;
    let r = (hex >> 16) as u8;
    [(r as f32) / 255.0, (g as f32) / 255.0, (b as f32) / 255.0]
}

impl Searcher {
    pub fn load(blob: &[u8]) -> Result<Self, JsError> {
        // TODO: change csv to blob
        let s = std::str::from_utf8(blob)?;
        // return Err(JsError::new(s));

        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b',')
            .from_reader(s.as_bytes());

        let mut majors = vec![];
        let mut labs = vec![];
        let mut latents = vec![];

        for v in rdr.records() {
            let ref rec = v?;
            let row: PaintInfo = rec.deserialize(None)?;

            let rgb = hex_to_rgb(row.rgb);
            let lab = Lab::from_rgb_normalized(&rgb);
            let latent = float_rgb_to_latent(&rgb);

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

    pub fn search(
        &self,
        rgb: u32,
        max_mix: u32,
        limit: usize,
    ) -> Result<Vec<SearchResult>, JsError> {
        let mut candidates = FixedBitSet::with_capacity(self.majors.len());
        for (i, maj) in self.majors.iter().enumerate() {
            if maj.brand == "gunze" && maj.serie == "C" {
                unsafe {
                    candidates.insert_unchecked(i);
                }
            }
        }

        let mut ctx = SearchContext {
            candidates,
            limit,
            kdtree_nearest_max_qty: unsafe { NonZero::new_unchecked(1000) },
            mix_limit: limit,
            mix2_prec: 0.01,
            mix2_iter: 5,
        };

        let mut results = vec![];
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
                delta_e: delta_e,
                rgb: rgb,
                portions: portions,
            });
        }

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

        let li1: Vec<_> = li.iter().map(|x| x.i).collect();

        for rem in 1..=max_mix {
            let mut li1 = self.search_mix(ctx, &rgb_out, li1.as_slice(), rem, true, ctx.limit);

            for x in li.iter() {
                ctx.candidates.set(x.i, true);
            }

            results.append(&mut li1);
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
        exclude: bool,
        limit: usize,
    ) -> Vec<SearchMix> {
        let mut results = vec![];

        for i in is.iter() {
            ctx.candidates.set(*i, false);

            let mut li = match rem {
                0 => vec![],
                1 => self.do_search_mix(ctx, rgb_out, *i, &|ctx, rgb| {
                    let lab = Lab::from_rgb_normalized(&rgb);
                    self.search_nearest(ctx, &lab)
                        .map(|SearchNearest { i, delta_e }| SearchMix {
                            portions: smallvec![Portion { i, t: 1f32 }],
                            latent: self.latents[i],
                            delta_e,
                        })
                        .into_iter()
                        .collect()
                }),
                _ => self.do_search_mix(ctx, rgb_out, *i, &|ctx, rgb| {
                    let lab = Lab::from_rgb_normalized(&rgb);
                    let li = self.search_nearest_n(ctx, &lab);
                    let li: Vec<_> = li.into_iter().map(|x| x.i).collect();
                    self.search_mix(
                        ctx,
                        rgb,
                        li.as_slice(),
                        rem - 1,
                        false,
                        limit * 3 / is.len(),
                    )
                }),
            };

            results.append(&mut li);
            if !exclude {
                ctx.candidates.set(*i, true);
            }
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

        for k in 1..n {
            let t0 = k as f32 * dt;
            let latent_1: Latent =
                std::array::from_fn(|i| (latent_out[i] - t0 * latent_0[i]) / (1f32 - t0));
            let rgb_1 = latent_to_float_rgb(&latent_1);

            for SearchMix {
                mut portions,
                latent,
                ..
            } in search_next(ctx, &rgb_1)
            {
                let SearchMix2Portion { t, delta_e } =
                    self.search_mix2_portion(ctx, latent_0, &latent, &lab_out, t0, dt * 0.5f32);

                for e in portions.iter_mut() {
                    e.t *= 1f32 - t;
                }
                portions.push(Portion { t, i: i_0 });
                let mix = SearchMix {
                    portions,
                    latent,
                    delta_e,
                };

                if let Some(x) = results.iter_mut().find(|x| {
                    x.portions
                        .iter()
                        .zip(mix.portions.iter())
                        .all(|(a, b)| a.i == b.i)
                }) {
                    if delta_e < x.delta_e {
                        *x = mix;
                    }
                } else {
                    results.push(mix);
                }
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
                let latent = std::array::from_fn(|i| (1f32 - t) * latent_1[i] - t * latent_0[i]);

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
                return SearchMix2Portion {
                    t: t0,
                    delta_e: mind,
                };
            }
        }
    }

    fn search_nearest_n(&self, ctx: &SearchContext, lab: &Lab) -> Vec<SearchNearest> {
        let point = [lab.l, lab.a, lab.b];
        let nearest_n = self
            .kdtree
            .nearest_n::<SquaredEuclidean>(&point, ctx.kdtree_nearest_max_qty);

        let mut results: Vec<_> = nearest_n
            .into_iter()
            .filter_map(|x| {
                let i = x.item as usize;
                if ctx.candidates.contains(i) {
                    let lab = self.labs[x.item as usize];
                    let d = cie00::diff(point, lab);
                    Some(SearchNearest { i, delta_e: d })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by_key(|x| OrderedFloat(x.delta_e));
        results
    }

    fn search_nearest(&self, ctx: &SearchContext, lab: &Lab) -> Option<SearchNearest> {
        let point = [lab.l, lab.a, lab.b];
        let nearest_n = self
            .kdtree
            .nearest_n::<SquaredEuclidean>(&point, ctx.kdtree_nearest_max_qty);

        let mut mini = None;
        let mut mind = f32::MAX;
        for x in nearest_n.into_iter() {
            let i = x.item as usize;
            if ctx.candidates.contains(i) {
                // 按欧氏距离粗筛的点还要用CIE2000做一次筛选
                let lab = self.labs[x.item as usize];
                let d = cie00::diff(point, lab);
                if d < mind {
                    mini = Some(i);
                    mind = d;
                }
            }
        }

        mini.map(|i| SearchNearest { i, delta_e: mind })
    }
}
