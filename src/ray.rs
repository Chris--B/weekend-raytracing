
use crate::float3::{
    Float,
    Float3,
};

#[derive(Copy, Clone, Debug, Default)]
pub struct Ray {
    pub origin: Float3,
    pub dir:    Float3,
}

impl Ray {
    pub fn at_t(&self, t: Float) -> Float3 {
        self.origin + t * self.dir
    }
}

/// When you look at a window at a steep angle, it becomes a mirror.
/// This is a simple approximation to that by Christophe Schlick.
pub fn schlick(cosine: Float, refraction_index: Float) -> Float {
    let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
    r0 *= r0;
    r0 + (1.0 - r0)*(1.0 - cosine).powi(5)
}
