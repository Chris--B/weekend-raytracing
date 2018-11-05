
use crate::ray::Ray;
use crate::float3::*;
use crate::hitable::*;

pub struct ScatterResult {
    attenuation: Float3,
    scattered:   Ray,
}

pub trait Material: std::fmt::Debug {
    fn scatter(&self, ray_in: &Ray, record: &HitRecord) -> ScatterResult;
}

#[derive(Copy, Clone, Debug, Default)]
struct Lambertian {
    albedo: Float3,
}

impl Material for Lambertian {
    fn scatter(&self, ray_in: &Ray, record: &HitRecord) -> ScatterResult {
        let target = record.p + record.normal + Float3::random_in_sphere();
        ScatterResult {
            scattered:   Ray { origin: record.p, dir: target - record.p },
            attenuation: self.albedo,
        }
    }
}
