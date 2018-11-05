
use crate::ray::Ray;
use crate::float3::*;
use crate::hitable::*;

pub struct ScatterResult {
    pub attenuation: Float3,
    pub scattered:   Option<Ray>,
}

pub trait Material: std::fmt::Debug {
    fn scatter(&self, ray_in: &Ray, record: &HitRecord) -> ScatterResult;
}

#[derive(Copy, Clone, Debug, Default)]
struct Lambertian {
    pub albedo: Float3,
}

impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, record: &HitRecord) -> ScatterResult {
        let target = record.p + record.normal + Float3::random_in_sphere();
        let scattered = Some(Ray { origin: record.p, dir: target - record.p });
        ScatterResult {
            scattered,
            attenuation: self.albedo,
        }
    }
}
