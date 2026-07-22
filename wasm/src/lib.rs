mod search;
mod sim;
mod wasm;

pub type Rgb = [f32; 3];
pub type Latent = [f32; 7];

pub type BoxError = Box<dyn std::error::Error>;

pub fn hex_to_rgb(hex: u32) -> Rgb {
    let b = (hex >> 0) as u8;
    let g = (hex >> 8) as u8;
    let r = (hex >> 16) as u8;
    [(r as f32) / 255.0, (g as f32) / 255.0, (b as f32) / 255.0]
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
