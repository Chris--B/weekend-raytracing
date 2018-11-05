
use std::rc::Rc;

use crate::float3::*;
use crate::material::Material;
use crate::ray::Ray;

#[derive(Clone, Default, Debug)]
pub struct HitRecord {
    // t-value of hit.
    pub t: Float,
    // Point in 3D Space of hit.
    pub p: Float3,
    // Normal value at point of hit.
    pub normal: Float3,
    // Material of hit.
    pub material: Option<Rc<dyn Material>>
}

pub trait Hitable {
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord>;
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Sphere {
    pub center: Float3,
    pub radius: Float,
}

impl Hitable for Sphere {
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        assert!(self.radius > 0.0);

        let oc = ray.origin - self.center;
        let a = ray.dir.length_sq();
        let b = oc.dot(&ray.dir);
        let c = oc.length_sq() - self.radius*self.radius;
        let discriminant = b*b - a*c;

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
                let p      = ray.at_t(t);
                // Make sure `normal` stays normal.
                let normal = (p - self.center) / self.radius;
                return Some(HitRecord { t, p, normal, material: None });
            }
            // It wasn't - check if the second one is.
            let t = (-b + discriminant.sqrt()) / a;
            if t_min < t && t < t_max {
                let p      = ray.at_t(t);
                // Make sure `normal` stays normal.
                let normal = (p - self.center) / self.radius;
                return Some(HitRecord { t, p, normal, material: None });
            }
        }
        // Nothing worked - no hit.
        None
    }
}

#[derive(Default)]
pub struct HitableList {
    pub hitables: Vec<Box<dyn Hitable>>,
}

impl Hitable for HitableList {
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let mut hit_record = HitRecord::default();
        let mut any_hit = false;
        let mut closest = t_max;

        for hitable in self.hitables.iter() {
            if let Some(new_record) = hitable.hit(ray, t_min, closest) {
                any_hit = true;
                closest = new_record.t;
                hit_record = new_record;
            }
        }

        if any_hit {
            Some(hit_record)
        } else {
            None
        }
    }
}