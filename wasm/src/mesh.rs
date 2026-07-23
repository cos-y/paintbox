use glam::Vec3;
use lab::Lab;
use libm::powf;

#[derive(Debug)]
pub struct Mesh {
    pub positions: Vec<f32>,
    pub colors: Vec<f32>,
    pub indices: Vec<u32>,
}

#[inline(always)]
fn srgb_to_linear(x: f32) -> f32 {
    if x >= 0.04045 {
        powf((x + 0.055) / 1.055, 2.4)
    } else {
        x / 12.92
    }
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            positions: vec![],
            colors: vec![],
            indices: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.colors.clear();
        self.indices.clear();
    }

    pub fn add(&mut self, vertices: &[Vec3], triangles: &[usize]) -> &mut Self {
        let i0 = self.positions.len() as u32 / 3;

        for xs in vertices.iter() {
            let [l, a, b] = xs.to_array();
            self.positions.push(l);
            self.positions.push(a);
            self.positions.push(b);
            let lab = Lab { l, a, b };
            let srgb = lab.to_rgb_normalized();
            let [r, g, b] = srgb.map(srgb_to_linear);
            self.colors.push(r);
            self.colors.push(g);
            self.colors.push(b);
        }

        for i in triangles.iter() {
            self.indices.push(i0 + *i as u32);
        }

        self
    }
}
