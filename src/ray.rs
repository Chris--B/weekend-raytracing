
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