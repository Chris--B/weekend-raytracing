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

#[derive(Debug)]
pub struct CameraInfo {
    pub lookfrom: Float3,
    pub lookat:   Float3,
    pub up:       Float3,
    pub vfov:     Float,
    pub aspect:   Float,
}

impl Camera {
    pub fn new(info: CameraInfo) -> Camera {
        let theta:  Float = info.vfov * std::f64::consts::PI / 180.0;
        let scale:  Float = (info.lookfrom - info.lookat).length();
        let height: Float = scale * (theta / 2.0).tan();
        let width:  Float = info.aspect * height;

        // Our orthonormal basis
        let w: Float3 = (info.lookfrom - info.lookat).unit();
        let u: Float3 = info.up.cross(&w).unit();
        let v: Float3 = w.cross(&u);

        Camera {
            origin:     info.lookfrom,
            horizontal: width * u,
            vertical:   height * v,
            lower_left: info.lookat - 0.5 * (width * u + height * v),
        }
    }

    pub fn get_ray(&self, s: Float, t: Float) -> Ray {
        let origin = self.origin;
        let dir = (self.lower_left - self.origin) +
                  (s*self.horizontal + t*self.vertical);
        Ray { origin, dir }
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
