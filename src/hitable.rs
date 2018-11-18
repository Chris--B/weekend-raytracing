use std::sync::Arc;

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
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord>;
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
        let sphere = sphere;
        sphere.hit(ray, t_min, t_max)
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
}
