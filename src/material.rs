
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
    pub fuzz:   Float,
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
        let dir = reflected + self.fuzz * Float3::random_in_sphere();
        *scattered   = Ray { origin: record.p, dir };
        (scattered.dir.dot(&record.normal) > 0.0)
    }
}

// Glass ball
#[derive(Copy, Clone, Debug, Default)]
pub struct Dielectric {
    pub refraction_index: Float,
}

impl Material for Dielectric {
    fn scatter(&self,
               ray_in:      &Ray,
               record:      &HitRecord,
               attenuation: &mut Float3,
               scattered:   &mut Ray)
        -> bool
    {
        // Our material doesn't attenutate anything.
        *attenuation = Float3::xyz(1, 1, 0); // Killing the blue channel!
        let reflected = ray_in.dir.reflect(record.normal);

        // We handle refraction differently depending on whether the ray
        // comes from inside or outside of the object.
        let outward_normal:   Float3;
        let refraction_index: Float;
        if ray_in.dir.dot(&record.normal) > 0.0 {
            outward_normal = -record.normal;
            refraction_index = self.refraction_index;
        } else {
            outward_normal = record.normal;
            refraction_index = 1.0 / self.refraction_index;
        }

        // We scatter the ray along one of the refracted or reflected paths.
        // Which one is determined by whether we can refract the incoming
        // ray against the surface we just hit.
        // If we can refract, we do.
        if let Some(refracted) = ray_in.dir.refract(outward_normal,
                                                    refraction_index)
        {
            *scattered = Ray { origin: record.p, dir: refracted };
            true
        } else {
            *scattered = Ray { origin: record.p, dir: reflected };
            false
        }
    }
}