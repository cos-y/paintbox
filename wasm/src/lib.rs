mod searcher;

use std::sync::Mutex;

use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;

use crate::searcher::{FilterOptions, Searcher};

#[wasm_bindgen]
pub fn color_diff(rgb_a: u32, rgb_b: u32) -> f32 {
    searcher::color_diff(rgb_a, rgb_b)
}

static SEARCHER: Lazy<Mutex<Option<Searcher>>> = Lazy::new(|| Mutex::new(None));

#[wasm_bindgen]
pub fn init_searcher(blob: &[u8], equiv_blob: &[u8]) -> Result<(), JsError> {
    let mut searcher = SEARCHER.lock()?;
    if let None = *searcher {
        *searcher = Some(Searcher::load(blob, equiv_blob)?);
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
pub fn search(rgb: u32, max_mix: u32, limit: usize, filter: JsValue) -> Result<JsValue, JsError> {
    let searcher = SEARCHER.lock()?;
    if let Some(ref searcher) = *searcher {
        let filter: FilterOptions = serde_wasm_bindgen::from_value(filter)?;
        let r = searcher.search(rgb, max_mix, limit, &filter)?;
        let r = serde_wasm_bindgen::to_value(&r)?;
        Ok(r)
    } else {
        Ok(JsValue::null())
    }
}
