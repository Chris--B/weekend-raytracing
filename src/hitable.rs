
use crate::float3::{
    Float,
    Float3,
};
use crate::ray::Ray;

#[derive(Copy, Clone, Default, Debug)]
struct HitRecord {
    // t-value of hit.
    t: Float,
    // Point in 3D Space of hit.
    p: Float3,
    // Normal value at point of hit.
    normal: Float3,
}

trait Hitable {
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord>;
}

#[derive(Copy, Clone, Default, Debug)]
struct Sphere {
    center: Float3,
    radius: Float,
}

impl Hitable for Sphere {
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.dir.length_sq();
        let b = oc.dot(&ray.dir);
        let c = oc.length_sq() - self.radius*self.radius;
        let discriminant = b*b - a*c;

        // There are three cases to consider here:
        //      1. discriminant < 0  => There are zero real solutions, no hit.
        //      2. discriminant == 0 => There is exactly one real solutioin,
        //          and the ray just barely grazes the sphere.
        //          We'll call that a "miss"
        //      3. discriminant > 0  => There are two real solutions, so the ray
        //          intersects the sphere and we need to hande the coloring.
        if discriminant > 0.0 {
            // Check that the first hit is within bounds.
            let temp = (-b - discriminant.sqrt()) / a;
            if t_min < temp && temp < t_max {
                let t      = temp;
                let p      = ray.at_t(t);
                let normal = (p - self.center) / self.radius;
                return Some(HitRecord { t, p, normal });
            }
            // It wasn't - check if the second one is.
            let temp = (-b + discriminant.sqrt()) / a;
            if t_min < temp && temp < t_max {
                let t      = temp;
                let p      = ray.at_t(t);
                let normal = (p - self.center) / self.radius;
                return Some(HitRecord { t, p, normal });
            }
        }
        // Nothing worked - no hit.
        None
    }
}

#[derive(Default)]
struct HitableList {
    hitables: Vec<Box<dyn Hitable>>,
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