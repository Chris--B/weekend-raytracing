
use crate::ray::Ray;
use crate::float3::*;
use crate::hitable::*;

pub trait Material: std::fmt::Debug {
    fn scatter(&self,
               ray_in:      &Ray,
               record:      &HitRecord,
               attenuation: &mut Float3,
               scattered:   &mut Ray)
        -> bool;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Lambertian {
    pub albedo: Float3,
}

impl Material for Lambertian {
    fn scatter(&self,
               _ray_in:     &Ray,
               record:      &HitRecord,
               attenuation: &mut Float3,
               scattered:   &mut Ray)
        -> bool
    {
        let target = record.p + record.normal + Float3::random_in_sphere();
        *attenuation = self.albedo;
        *scattered   = Ray { origin: record.p, dir: target - record.p };
        true
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Metal {
    pub albedo: Float3,
}

impl Material for Metal {
    fn scatter(&self,
               ray_in:      &Ray,
               record:      &HitRecord,
               attenuation: &mut Float3,
               scattered:   &mut Ray)
        -> bool
    {
        let reflected = ray_in.dir.unit().reflect(record.normal);
        *attenuation = self.albedo;
        *scattered   = Ray { origin: record.p, dir: reflected };
        (scattered.dir.dot(&record.normal) > 0.0)
    }
}
