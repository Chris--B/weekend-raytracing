use std::{
    mem,
    sync::Arc,
};

use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct HitRecord {
    // t-value of hit.
    pub t: Float,
    // Point in 3D Space of hit.
    pub p: Float3,
    // Normal value at point of hit.
    pub normal: Float3,
    // Material of hit.
    pub material: Arc<dyn Material>,
}

pub trait Hitable: std::fmt::Debug + Send + Sync {
    /// Compute whether and where a ray intersections this object.
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord>;

    /// Compute the bounding box for this object.
    fn bounding_box(&self, t0: Float, t1: Float) -> Option<Aabb>;
}

#[derive(Clone, Debug)]
pub struct Sphere {
    pub center: Float3,
    pub radius: Float,
    pub material: Arc<dyn Material>,
}

impl Hitable for Sphere {
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.dir.length_sq();
        let b = oc.dot(&ray.dir);
        let c = oc.length_sq() - self.radius * self.radius;
        let discriminant = b * b - a * c;

        // There are three cases to consider here:
        //      1. discriminant < 0  => There are zero real solutions, no hit.
        //      2. discriminant == 0 => There is exactly one real solutioin,
        //          and the ray just barely grazes the sphere.
        //      3. discriminant > 0  => There are two real solutions, so the ray
        //          intersects the sphere and we need to hande the coloring.
        if discriminant >= 0.0 {
            // Check that the first hit is within bounds.
            let t = (-b - discriminant.sqrt()) / a;
            if t_min < t && t < t_max {
                let p = ray.at_t(t);
                // Make sure `normal` stays normal.
                let normal = (p - self.center) / self.radius;
                let material = self.material.clone();
                return Some(HitRecord { t, p, normal, material });
            }
            // It wasn't - check if the second one is.
            let t = (-b + discriminant.sqrt()) / a;
            if t_min < t && t < t_max {
                let p = ray.at_t(t);
                // Make sure `normal` stays normal.
                let normal = (p - self.center) / self.radius;
                let material = self.material.clone();
                return Some(HitRecord { t, p, normal, material });
            }
        }
        // Nothing worked - no hit.
        None
    }

    // This object does not move wrt time, so we ignore the time inputs.
    fn bounding_box(&self, _t0: Float, _t1: Float) -> Option<Aabb> {
        let r = Float3::xxx(self.radius).abs();
        Some(Aabb {
            min: self.center - r,
            max: self.center + r,
        })
    }
}

#[derive(Clone, Debug)]
pub struct MovingSphere {
    // Static geometry and material. This represents the `MovingSphere` at t=0.
    pub sphere: Sphere,
    // The motion vector. The sphere is centered at `sphere.center + t * motion`.
    pub motion: Float3,
}

impl Hitable for MovingSphere {
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        // There's no easy way to
        //      1) reuse the sphere code, and
        //      2) not clone the material
        // Thankfully, an Rc::clone() is just a ref count increment.
        // When we switch to Arcs, the ref count increment becomes atomic.
        let mut sphere = self.sphere.clone();
        sphere.center += ray.t * self.motion;
        sphere.hit(ray, t_min, t_max)
    }

    fn bounding_box(&self, t0: Float, t1: Float) -> Option<Aabb> {
        // This is less than ideal. See the comment in
        // <MovingSphere as Hitable>::hit() for details.
        let mut sphere_t0 = self.sphere.clone();
        sphere_t0.center += self.motion * t0;

        let mut sphere_t1 = self.sphere.clone();
        sphere_t1.center += self.motion * t1;

        // Spheres always have a bounding box, so these unwraps never panic.
        Some(Aabb::surrounding(&sphere_t0.bounding_box(t0, t1).unwrap(),
                               &sphere_t1.bounding_box(t0, t1).unwrap()))
    }
}

#[derive(Debug, Default)]
pub struct HitableList {
    pub hitables: Vec<Box<dyn Hitable>>,
}

impl Hitable for HitableList {
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let mut o_hit_record = None;
        let mut closest = t_max;

        for hitable in self.hitables.iter() {
            if let Some(new_record) = hitable.hit(ray, t_min, closest) {
                closest = new_record.t;
                o_hit_record = Some(new_record);
            }
        }

        o_hit_record
    }

    fn bounding_box(&self, t0: Float, t1: Float) -> Option<Aabb> {
        // Iterate over the bounding boxes of `self.hitables`.
        let mut iter = self.hitables.iter().map(|h| h.bounding_box(t0, t1));

        // TODO: Can this be cleaner?
        // We can only compute a bounding box if
        //      1) There are items in the list
        let mut running_aabb: Aabb;
        if let Some(o_aabb) = iter.next() {
            //  2) Every item in the list has a bounding box.
            if let Some(aabb) = o_aabb {
                running_aabb = aabb;
            } else {
                return None;
            }
        } else {
            return None;
        }

        for o_next_box in iter {
            if let Some(next_box) = o_next_box {
                running_aabb = Aabb::surrounding(&running_aabb, &next_box);
            } else {
                // Even a single thing without a bounding box stops us from
                // being able to bound all of its parent lists.
                return None;
            }
        }

        Some(running_aabb)
    }
}

pub struct Aabb {
    pub min: Float3,
    pub max: Float3,
}

impl Aabb {
    pub fn surrounding(box0: &Aabb, box1: &Aabb) -> Aabb {
        Aabb {
            min: box0.min.min(&box1.min),
            max: box0.max.max(&box1.max),
        }
    }

    pub fn hit(&self, ray: &Ray, tmin: Float, tmax: Float) -> bool {
        let inv_dir: Float3 = 1.0 / ray.dir;

        let mut t0 = (self.min - ray.origin) * inv_dir;
        let mut t1 = (self.max - ray.origin) * inv_dir;

        // Reverse the interval along the axis when the ray is going "backwards"
        if inv_dir.x < 0.0 { mem::swap(&mut t0.x, &mut t1.x); }
        if inv_dir.y < 0.0 { mem::swap(&mut t0.y, &mut t1.y); }
        if inv_dir.z < 0.0 { mem::swap(&mut t0.z, &mut t1.z); }

        t0 = t0.min(&Float3::xxx(tmin));
        t1 = t1.max(&Float3::xxx(tmax));

        (t0 < t1)
    }
}
