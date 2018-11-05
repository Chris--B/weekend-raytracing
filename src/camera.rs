
use std::default;

use crate::float3::*;
use crate::ray::Ray;

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub origin:     Float3,
    pub lower_left: Float3,
    pub horizontal: Float3,
    pub vertical:   Float3,
}

impl Camera {
    pub fn get_ray(&self, u: Float, v: Float) -> Ray {
        Ray {
            origin: self.origin,
            dir:    self.lower_left + u*self.horizontal + v*self.vertical,
        }
    }
}

impl default::Default for Camera {
    fn default() -> Camera {
        Camera {
            origin:     Float3::default(),
            lower_left: Float3::xyz(-2.0, -1.0, -1.0),
            horizontal: Float3::xyz(4.0, 0.0, 0.0),
            vertical:   Float3::xyz(0.0, 2.0, 0.0),
        }
    }
}
