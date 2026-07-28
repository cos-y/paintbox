mod brute;
mod fast;

use std::sync::{Arc, Mutex};

use crate::{BoxError, Rgb, mesh::Mesh};

pub use brute::BruteHull;
pub use fast::FastHull;

pub trait Hull {
    fn new(colors: Vec<Rgb>) -> Result<Box<Self>, BoxError>
    where
        Self: Sized;

    fn insert(&mut self, color: Rgb);

    fn mesh(&self) -> Arc<Mutex<Mesh>>;
}
