use lab::Lab;
use mixbox::{float_rgb_to_latent, latent_to_float_rgb};
use oklab::*;

pub use oklab::{Oklab, Rgb};

pub mod gamut;
pub mod search;
pub mod tess;

mod wasm;

pub type Latent = [f32; 7];

pub type BoxError = Box<dyn std::error::Error>;

#[inline]
pub fn hex_to_rgb(hex: u32) -> Rgb<f32> {
    let b = (hex >> 0) as u8;
    let g = (hex >> 8) as u8;
    let r = (hex >> 16) as u8;
    Rgb::new((r as f32) / 255.0, (g as f32) / 255.0, (b as f32) / 255.0)
}

#[inline]
pub fn rgb_to_lab(rgb: Rgb<f32>) -> Lab {
    Lab::from_rgb_normalized(&[rgb.r, rgb.g, rgb.b])
}

#[inline]
pub fn rgb_to_oklab(rgb: Rgb<f32>) -> Oklab {
    srgb_f32_to_oklab(rgb)
}

#[inline]
pub fn oklab_to_rgb(oklab: Oklab) -> Rgb<f32> {
    oklab_to_srgb_f32(oklab)
}

#[inline]
pub fn rgb_to_latent(rgb: Rgb<f32>) -> Latent {
    float_rgb_to_latent(&[rgb.r, rgb.g, rgb.b])
}

#[inline]
pub fn oklab_dist(a: Oklab, b: Oklab) -> f32 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    (dl * dl + da * da + db * db).sqrt()
}

#[inline]
pub fn latent_to_rgb(latent: &Latent) -> Rgb<f32> {
    let [r, g, b] = latent_to_float_rgb(latent);
    Rgb { r, g, b }
}

pub fn lerp_latent(l0: &Latent, l1: &Latent, t: f32) -> Latent {
    std::array::from_fn(|i| t * l0[i] + (1f32 - t) * l1[i])
}

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&format!($($t)*).into());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            println!($($t)*);
        }
    };
}
