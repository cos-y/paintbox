use std::{collections::HashMap, sync::Arc};

use glam::Vec2;

#[derive(Clone)]
pub struct Tessellation {
    pub uvs: Arc<Vec<(f32, f32)>>,
    pub triangles: Arc<Vec<usize>>,
}

pub struct Tessellator {
    lookup: HashMap<[usize; 3], Tessellation>,
}

impl Tessellator {
    pub fn new() -> Self {
        Tessellator {
            lookup: HashMap::new(),
        }
    }

    pub fn get(&mut self, key: &[usize; 3]) -> Tessellation {
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
        let [a, b, c] = *key;

        let mut uvs = vec![];
        let mut triangles = vec![];

        let n = (a + b + c) / 3 - 1;
        if n > 1 {
            let dx = 1f32 / (n + 1) as f32;
            for i in 1..n {
                for j in 1..=(n - i) {
                    let u = i as f32 * dx;
                    let v = j as f32 * dx;
                    uvs.push((u, v));
                }
            }

            let mut k = 0;
            for i in 1..(n - 1) {
                for j in 1..(n - i) {
                    let a = j + k;
                    let b = a - 1;
                    let c = b + n - i;

                    triangles.push(a);
                    triangles.push(b);
                    triangles.push(c);

                    if i > 1 {
                        let d = b + i - n;

                        triangles.push(a);
                        triangles.push(d);
                        triangles.push(b);
                    }
                }

                k += n - i;
            }
        }

        let ic = uvs.len();
        for i in 0..c {
            let t = i as f32 / c as f32;
            uvs.push((t, 0.0));
        }

        let ib = uvs.len();
        for i in 0..b {
            let t = i as f32 / b as f32;
            uvs.push((0.0, 1.0 - t));
        }

        let ia = uvs.len();
        for i in 0..a {
            let t = i as f32 / a as f32;
            uvs.push((1.0 - t, t));
        }

        fn seal(
            triangles: &mut Vec<usize>,
            n: usize,
            m: usize,
            ns: impl Iterator<Item = usize>,
            ms: impl Iterator<Item = usize>,
        ) {
            let mut xi = 0f32;
            let mut xj = 0f32;

            let mut ns = ns.peekable();
            let mut ms = ms.peekable();

            let mut i = ns.next().unwrap();
            let mut j = ms.next().unwrap();

            let di = 1f32 / (n - 1).max(1) as f32;
            let dj = 1f32 / (m - 1).max(1) as f32;

            while ns.peek().is_some() || ms.peek().is_some() {
                let xi_1 = xi + di;
                let xj_1 = xj + dj;

                triangles.push(i);
                triangles.push(j);

                triangles.push(if xi_1 < xj_1 {
                    xi = xi_1;
                    i = ns.next().unwrap();
                    i
                } else {
                    xj = xj_1;
                    j = ms.next().unwrap();
                    j
                });
            }
        }

        fn tri(triangles: &mut Vec<usize>, a: usize, b: usize, c: usize) {
            triangles.push(a);
            triangles.push(b);
            triangles.push(c);
        }

        fn quad(
            triangles: &mut Vec<usize>,
            uvs: &Vec<(f32, f32)>,
            i: usize,
            j: usize,
            a: usize,
            b: usize,
        ) {
            let (xa, ya) = uvs[a];
            let (xb, yb) = uvs[b];
            let (xj, yj) = uvs[j];

            let va = Vec2::new(xa, ya);
            let vb = Vec2::new(xb, yb);
            let vj = Vec2::new(xj, yj);

            let vja = va - vj;
            let vjb = vb - vj;

            if vja.perp_dot(vjb) < 0.0 {
                tri(triangles, i, a, b);
                tri(triangles, j, b, a);
            } else {
                tri(triangles, i, j, b);
                tri(triangles, j, i, a);
            }
        }

        if n > 1 {
            let m = n * (n - 1) / 2 - 1;
            let u = |i: usize| i * (2 * n - i - 1) / 2;
            let v = |i: usize| n - 2 - i;
            let w = |i: usize| m - i * (i + 1) / 2;

            if c > 1 {
                let ui = (0..(n - 1)).map(u);
                seal(&mut triangles, n - 1, c - 1, ui, ic + 1..ic + c);
            } else {
                tri(&mut triangles, ic, ia, u(0));
                for i in 1..n - 1 {
                    tri(&mut triangles, u(i), u(i - 1), ia);
                }
            }

            if b > 1 {
                let vi = (0..(n - 1)).map(v);
                seal(&mut triangles, n - 1, b - 1, vi, ib + 1..ib + b);
            } else {
                tri(&mut triangles, ib, ic, v(0));
                for i in 1..n - 1 {
                    tri(&mut triangles, v(i), v(i - 1), ic);
                }
            }

            if a > 1 {
                let m = n * (n - 1) / 2 - 1;
                let wi = (0..(n - 1)).map(w);
                seal(&mut triangles, n - 1, a - 1, wi, ia + 1..ia + a);
            } else {
                tri(&mut triangles, ia, ib, w(0));
                for i in 1..n - 1 {
                    tri(&mut triangles, w(i), w(i - 1), ib);
                }
            }

            if b > 1 && c > 1 {
                quad(&mut triangles, &uvs, ic, u(0), ic + 1, ib + b - 1);
            } else if b > 1 {
                tri(&mut triangles, ic, u(0), ib + b - 1);
            } else if c > 1 {
                tri(&mut triangles, ic, ic + 1, u(0));
            }

            if c > 1 && a > 1 {
                quad(&mut triangles, &uvs, ia, w(0), ia + 1, ic + c - 1);
            } else if c > 1 {
                tri(&mut triangles, ia, w(0), ic + c - 1);
            } else if a > 1 {
                tri(&mut triangles, ia, ia + 1, w(0));
            }

            if a > 1 && b > 1 {
                quad(&mut triangles, &uvs, ib, v(0), ib + 1, ia + a - 1);
            } else if a > 1 {
                tri(&mut triangles, ib, v(0), ia + a - 1);
            } else if b > 1 {
                tri(&mut triangles, ib, ib + 1, v(0));
            }
        } else {
            if a > 1 && b > 1 && c > 1 {
                tri(&mut triangles, ic, ic + 1, ib + b - 1);
                tri(&mut triangles, ia, ia + 1, ic + c - 1);
                tri(&mut triangles, ib, ib + 1, ia + a - 1);

                if a + b + c == 6 {
                    tri(&mut triangles, ic + 1, ia + 1, ib + 1);
                } else {
                    let mut li = vec![];
                    for i in ia + 1..ia + a {
                        li.push(i);
                    }
                    for i in ib + 1..ib + b {
                        li.push(i);
                    }
                    for i in ic + 1..ic + c {
                        li.push(i);
                    }
                    for i in 1..li.len() - 1 {
                        tri(&mut triangles, li[0], li[i], li[i + 1]);
                    }
                }
            } else {
                let li = [(a, ia), (b, ib), (c, ic)];
                let i = li
                    .iter()
                    .enumerate()
                    .find_map(|(i, x)| if x.0 == 1 { Some(i) } else { None })
                    .unwrap();
                let (a, ia) = li[i];
                let (b, ib) = li[(i + 1) % 3];
                let (c, ic) = li[(i + 2) % 3];
                for i in ib..ib + b - 1 {
                    tri(&mut triangles, ia, i, i + 1);
                }
                for i in ic..ic + c - 1 {
                    tri(&mut triangles, i, i + 1, ib + b - 1);
                }
                tri(&mut triangles, ia, ib + b - 1, ic + c - 1);
            }
        }

        Tessellation {
            uvs: Arc::new(uvs),
            triangles: Arc::new(triangles),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{log, tess::Tessellator};

    #[test]
    pub fn test_tess_1x1x1() {
        let tess = Tessellator::new().get(&[1, 1, 1]);
        assert_eq!(*tess.triangles, vec![0, 2, 1]);
    }

    #[test]
    pub fn test_tess_2x2x1() {
        let tess = Tessellator::new().get(&[2, 2, 1]);
        assert_eq!(*tess.triangles, vec![0, 3, 4, 1, 2, 4, 0, 4, 2]);
    }

    #[test]
    pub fn test_tess_3x1x1() {
        let tess = Tessellator::new().get(&[3, 1, 1]);
        assert_eq!(*tess.triangles, vec![2, 3, 0, 3, 4, 0, 1, 0, 4]);
    }

    #[test]
    pub fn test_tess_2x2x2() {
        let tess = Tessellator::new().get(&[2, 2, 2]);
        assert_eq!(*tess.triangles, vec![0, 1, 3, 4, 5, 1, 2, 3, 5, 1, 5, 3]);
    }

    #[test]
    pub fn test_tess_2x2x3() {
        let tess = Tessellator::new().get(&[2, 2, 3]);
        assert_eq!(
            *tess.triangles,
            vec![0, 1, 4, 5, 6, 2, 3, 4, 6, 6, 4, 1, 6, 1, 2]
        );
    }

    #[test]
    pub fn test_tess_2x3x3() {
        let tess = Tessellator::new().get(&[2, 3, 3]);
        assert_eq!(
            *tess.triangles,
            vec![0, 1, 5, 6, 7, 2, 3, 4, 7, 7, 4, 5, 7, 5, 1, 7, 1, 2]
        );
    }

    #[test]
    pub fn test_tess_3x3x3() {
        let tess = Tessellator::new().get(&[3, 3, 3]);
        assert_eq!(
            *tess.triangles,
            vec![
                0, 2, 3, 0, 5, 6, 0, 8, 9, 1, 2, 6, 0, 6, 2, 7, 8, 3, 0, 3, 8, 4, 5, 9, 0, 9, 5
            ]
        );
    }

    #[test]
    pub fn test_tess_4x3x3() {
        let tess = Tessellator::new().get(&[4, 3, 3]);
        assert_eq!(
            *tess.triangles,
            vec![
                0, 2, 3, 0, 5, 6, 0, 8, 9, 0, 9, 10, 1, 2, 6, 0, 6, 2, 7, 8, 3, 0, 3, 8, 4, 5, 10,
                0, 10, 5
            ]
        );
    }

    #[test]
    pub fn test_tess_4x4x2() {
        let tess = Tessellator::new().get(&[4, 4, 2]);
        assert_eq!(
            *tess.triangles,
            vec![
                0, 4, 5, 0, 5, 6, 0, 8, 9, 0, 9, 10, 1, 2, 6, 0, 6, 2, 7, 8, 2, 0, 2, 8, 3, 4, 10,
                0, 10, 4
            ]
        );
    }

    #[test]
    pub fn test_tess_4x4x1() {
        let tess = Tessellator::new().get(&[4, 4, 1]);
        assert_eq!(
            *tess.triangles,
            vec![
                1, 6, 0, 0, 3, 4, 0, 4, 5, 0, 7, 8, 0, 8, 9, 1, 0, 5, 6, 7, 0, 2, 3, 9, 0, 9, 3
            ]
        );
    }
}
