use std::f64::consts;

use crate::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub w:           Float3,
    pub u:           Float3,
    pub v:           Float3,
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
    pub focus_dist: Float,
}

impl Camera {
    pub fn new(info: CameraInfo) -> Camera {
        // We need a few things to create our camera.
        // Ultimately, we want a plane and an origin. We'll fire rays from the
        // origin at points in the plane and then into the scene.

        // We determine the height of the plane using the vertical field of view
        // passed in through `info`.
        // To keep the top and bottom lines of the plane at the same visual
        // angle, we scale its height by the distance away that it is.
        // Width then follows from `info.aspect`.
        let theta:       Float = info.vfov * consts::PI / 180.0;
        let half_height: Float = (theta / 2.0).tan();
        let half_width:  Float = info.aspect * half_height;

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
        let v: Float3 = w.cross(&u); // Note: Don't need to `.unit()`

        let CameraInfo { lookfrom, focus_dist, ..} = info;
        Camera {
            u,
            v,
            w,
            origin:      lookfrom,
            horizontal:  2.0 * focus_dist * half_width * u,
            vertical:    2.0 * focus_dist * half_height * v,
            lower_left:  lookfrom
                         - focus_dist * (half_width * u + half_height * v + w),
            lens_radius: info.aperature / 2.0,
        }
    }

    pub fn get_ray(&self, s: Float, t: Float) -> Ray {
        let disk = self.lens_radius * random_in_disk();
        let offset = self.u * disk.x + self.v * disk.y;
        let dir = (self.lower_left - self.origin) +
                  (s*self.horizontal + t*self.vertical);
        Ray {
            origin: self.origin + offset,
            dir:    dir - offset,
        }
    }
}
