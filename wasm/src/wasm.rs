use std::sync::Mutex;

use empfindung::cie00;
use lab::Lab;
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;
use web_sys::js_sys::{Float32Array, Uint32Array};

use crate::{
    BoxError, hex_to_rgb,
    hull::Hull,
    search::{FilterOptions, Searcher},
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

    pub fn points(&self) -> Float32Array {
        unsafe { Float32Array::view(&self.0.mesh().positions) }
    }

    pub fn colors(&self) -> Float32Array {
        unsafe { Float32Array::view(&self.0.mesh().colors) }
    }

    pub fn indices(&self) -> Uint32Array {
        unsafe { Uint32Array::view(&self.0.mesh().indices) }
    }
}

#[wasm_bindgen]
pub fn hull(li: &[u32]) -> Result<HullProxy, JsError> {
    HullProxy::new(li)
}
