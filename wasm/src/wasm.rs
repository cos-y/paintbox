use std::sync::{Arc, Mutex};

use empfindung::{ToLab, cie00};
use glam::Vec3;
use lab::Lab;
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;
use web_sys::js_sys::{Float32Array, Uint32Array};

use crate::{
    BoxError, Rgb, hex_to_rgb,
    hull::Hull,
    mesh::Mesh,
    search::{FilterOptions, Searcher},
    tess::get_triangle_tesselation,
};

static SEARCHER: Lazy<Mutex<Option<Searcher>>> = Lazy::new(|| Mutex::new(None));

fn to_jserr(e: BoxError) -> JsError {
    JsError::new(&e.to_string())
}

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn color_diff(rgb_a: u32, rgb_b: u32) -> f32 {
    let lab_a = Lab::from_rgb_normalized(&hex_to_rgb(rgb_a));
    let lab_b = Lab::from_rgb_normalized(&hex_to_rgb(rgb_b));
    cie00::diff(&lab_a, &lab_b)
}

#[wasm_bindgen]
pub fn init_searcher(blob: &[u8], equiv_blob: &[u8]) -> Result<(), JsError> {
    let mut searcher = SEARCHER.lock()?;
    if let None = *searcher {
        *searcher = Some(Searcher::load(blob, equiv_blob).map_err(to_jserr)?);
    }
    Ok(())
}

#[wasm_bindgen]
pub fn find_direct_equivalences(index: usize) -> Result<JsValue, JsError> {
    let searcher = SEARCHER.lock()?;
    if let Some(ref searcher) = *searcher {
        let r = serde_wasm_bindgen::to_value(&searcher.direct_equivalences(index))?;
        Ok(r)
    } else {
        Ok(JsValue::null())
    }
}

#[wasm_bindgen]
pub fn list_paints() -> Result<JsValue, JsError> {
    let searcher = SEARCHER.lock()?;
    if let Some(ref searcher) = *searcher {
        let r = serde_wasm_bindgen::to_value(&searcher.list())?;
        Ok(r)
    } else {
        Ok(JsValue::null())
    }
}

#[wasm_bindgen]
pub fn search(rgb: u32, opts: JsValue) -> Result<JsValue, JsError> {
    let searcher = SEARCHER.lock()?;
    if let Some(ref searcher) = *searcher {
        let filter: FilterOptions = serde_wasm_bindgen::from_value(opts)?;
        let r = searcher.search(rgb, &filter).map_err(to_jserr)?;
        let r = serde_wasm_bindgen::to_value(&r)?;
        Ok(r)
    } else {
        Ok(JsValue::null())
    }
}

#[wasm_bindgen]
pub struct MeshProxy(Arc<Mutex<Mesh>>);

#[wasm_bindgen]
impl MeshProxy {
    pub fn positions(&self) -> Float32Array {
        let mesh = self.0.lock().unwrap();
        unsafe { Float32Array::view(&mesh.positions) }
    }

    pub fn colors(&self) -> Float32Array {
        let mesh = self.0.lock().unwrap();
        unsafe { Float32Array::view(&mesh.colors) }
    }

    pub fn indices(&self) -> Uint32Array {
        let mesh = self.0.lock().unwrap();
        unsafe { Uint32Array::view(&mesh.indices) }
    }
}

#[wasm_bindgen]
pub struct HullProxy(Hull);

#[wasm_bindgen]
impl HullProxy {
    pub fn new(rgbs: &[u32]) -> Result<Self, JsError> {
        Hull::new(rgbs.iter().map(|x| hex_to_rgb(*x)).collect())
            .map_err(to_jserr)
            .map(|v| Self(v))
    }

    pub fn add(&mut self, rgb: u32) {
        self.0.insert(hex_to_rgb(rgb));
    }

    pub fn mesh(&mut self) -> MeshProxy {
        MeshProxy(self.0.mesh.clone())
    }
}

#[wasm_bindgen]
pub fn get_hull(li: &[u32]) -> Result<HullProxy, JsError> {
    HullProxy::new(li)
}

#[wasm_bindgen]
pub fn get_srgb_mesh(n: usize) -> MeshProxy {
    let tess = get_triangle_tesselation(&[n, n, n]);

    let r = [1f32, 0f32, 0f32];
    let g = [0f32, 1f32, 0f32];
    let b = [0f32, 0f32, 1f32];
    let c = [0f32, 1f32, 1f32];
    let m = [1f32, 0f32, 1f32];
    let y = [1f32, 1f32, 0f32];
    let k = [0f32, 0f32, 0f32];
    let w = [1f32, 1f32, 1f32];

    let li = [k, g, y, r, b, c, w, m];
    // .map(|x| Lab::from_rgb_normalized(&x));
    let fs = [
        (0, 1, 2),
        (2, 3, 0),
        (4, 5, 6),
        (6, 7, 4),
        (1, 5, 6),
        (1, 2, 6),
        (2, 6, 7),
        (2, 3, 7),
        (0, 3, 7),
        (0, 7, 4),
        (0, 1, 4),
        (1, 4, 5),
    ];

    let mut mesh = Mesh::new();
    for (a, b, c) in fs {
        let vertices: Vec<_> = tess
            .uvs
            .iter()
            .map(|(u, v)| {
                let rgb: Rgb =
                    std::array::from_fn(|i| u * li[a][i] + v * li[b][i] + (1.0 - u - v) * li[c][i]);
                let (l, a, b) = Lab::from_rgb_normalized(&rgb).to_lab();
                Vec3::new(l, a, b)
            })
            .collect();
        mesh.add(&vertices, &tess.triangles);
    }

    MeshProxy(Arc::new(Mutex::new(mesh)))
}
