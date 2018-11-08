use crate::float3::*;
use crate::ray::Ray;

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub origin:      Float3,
    pub lower_left:  Float3,
    pub horizontal:  Float3,
    pub vertical:    Float3,
    pub lens_radius: Float,
}

#[derive(Debug)]
pub struct CameraInfo {
    pub lookfrom:   Float3,
    pub lookat:     Float3,
    pub up:         Float3,
    pub vfov:       Float,
    pub aspect:     Float,
    pub aperature:  Float,
}

impl Camera {
    pub fn new(info: CameraInfo) -> Camera {
        // We need a few things to create our camera.
        // Ultimately, we want a plane and an origin. We'll fire rays from the
        // origin at points in the plane and then into the scene.

        let lens_radius = info.aperature / 2.0;

        // We determine the height of the plane using the vertical field of view
        // passed in through `info`.
        // To keep the top and bottom lines of the plane at the same visual
        // angle, we scale its height by the distance away that it is.
        // Width then follows from `info.aspect`.
        let theta:      Float = info.vfov * std::f64::consts::PI / 180.0;
        let focal_dist: Float = (info.lookfrom - info.lookat).length();
        let height:     Float = focal_dist * (theta / 2.0).tan();
        let width:      Float = info.aspect * height;

        // We also need to construct three directions to describe the plane:
        //      w: The direction from the origin to the plane.
        //          In screenspace, *into* the image.
        //      u: The direction "right" of the camera.
        //          In screenspace, the *right* of the image.
        //      v: The direction "up" of the camera.
        //          In screenspace, the *top* of the image.
        // w and v should be co-planer (a "wedge" into the main plane)
        // u and v should be co-planer (defining normalized screen space)
        // Ultimately, this is just an orthonormal basis.
        let w: Float3 = (info.lookfrom - info.lookat).unit();
        let u: Float3 = info.up.cross(&w).unit();
        let v: Float3 = w.cross(&u); // Note: Don't need to `.unit()`!

        Camera {
            origin:      info.lookfrom,
            horizontal:  width * u,
            vertical:    height * v,
            // `info.lookat` bisects the main plane exactly in its center,
            // so width and height are halved.
            lower_left:  info.lookat - 0.5 * (width * u + height * v),
            lens_radius,
        }
    }

    pub fn get_ray(&self, s: Float, t: Float) -> Ray {
        let origin = self.origin;

        let disk = self.lens_radius * Float3::random_in_disk();
        let offset = s * disk.x + t * disk.y;
        let dir = (self.lower_left - self.origin) +
                  (s*self.horizontal + t*self.vertical) +
                  offset;

        Ray { origin, dir }
    }
}
