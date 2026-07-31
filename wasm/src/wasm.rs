use std::sync::Mutex;

use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;
use web_sys::js_sys::Float32Array;
use web_time::Instant;

use crate::{
    BoxError, hex_to_rgb, log, oklab_dist, rgb_to_oklab,
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
pub fn color_diff(a: u32, b: u32) -> f32 {
    let a = rgb_to_oklab(hex_to_rgb(a));
    let b = rgb_to_oklab(hex_to_rgb(b));
    oklab_dist(a, b)
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
pub struct Gamut(crate::gamut::Gamut);

#[wasm_bindgen]
impl Gamut {
    pub fn insert(&mut self, rgb: u32) -> bool {
        self.0.insert(hex_to_rgb(rgb))
    }

    pub fn insert_many(&mut self, rgbs: &[u32]) -> bool {
        timed(&format!(":: Gamut :: insert_many ({:?})", rgbs), || {
            self.0
                .insert_many(rgbs.iter().map(|x| hex_to_rgb(*x)).collect())
        })
    }

    pub fn matrices(&self) -> Float32Array {
        unsafe { Float32Array::view(&self.0.matrices) }
    }

    pub fn colors(&self) -> Float32Array {
        unsafe { Float32Array::view(&self.0.colors) }
    }
}

#[wasm_bindgen]
pub fn new_gamut(ndiv: usize, li: &[u32]) -> Result<Gamut, JsError> {
    timed(&format!(":: Gamut :: new ({}, {:?})", ndiv, li), || {
        let rgbs = li.iter().map(|x| hex_to_rgb(*x)).collect();
        let gamut = crate::gamut::Gamut::new(ndiv, rgbs).map_err(to_jserr)?;
        Ok(Gamut(gamut))
    })
}

fn timed<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let time = Instant::now();
    let result = f();
    let duration = time.elapsed();
    log!("{} => {} ms", label, duration.as_millis());
    result
}
