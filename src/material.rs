use crate::prelude::*;

pub trait Material: std::fmt::Debug + Send + Sync {
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


#[derive(Copy, Clone, Debug, Default)]
pub struct NormalToRgb {}

impl Material for NormalToRgb {
    fn scatter(&self,
               _ray_in:     &Ray,
               record:      &HitRecord,
               attenuation: &mut Float3,
               _scattered:  &mut Ray)
        -> bool
    {
        *attenuation = record.normal.unit();
        // No scattered ray.
        false
    }
}

impl Material for Lambertian {
    fn scatter(&self,
               ray_in:      &Ray,
               record:      &HitRecord,
               attenuation: &mut Float3,
               scattered:   &mut Ray)
        -> bool
    {
        let target = record.p + record.normal + random_in_sphere();
        *attenuation = self.albedo;
        *scattered = Ray {
            origin: record.p,
            dir:    target - record.p,
            t:      ray_in.t,
        };
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
        let dir = reflected + self.fuzz * random_in_sphere();
        *scattered = Ray {
            origin: record.p,
            dir,
            t: ray_in.t,
        };
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
        // Our material doesn't attenuate anything.
        *attenuation = Float3::xyz(1., 1., 1.);
        let reflected = ray_in.dir.reflect(record.normal);

        // We handle refraction differently depending on whether the ray
        // comes from inside or outside of the object.
        let outward_normal:   Float3;
        let refraction_index: Float;
        let cosine:           Float;
        if ray_in.dir.dot(&record.normal) > 0.0 {
            outward_normal = -record.normal;
            refraction_index = self.refraction_index;
            cosine = refraction_index * ray_in.dir.unit().dot(&record.normal);
        } else {
            outward_normal = record.normal;
            refraction_index = 1.0 / self.refraction_index;
            cosine = -ray_in.dir.unit().dot(&record.normal);
        }

        // We scatter the ray along one of the refracted or reflected paths.
        // Which one is determined by whether we can refract the incoming
        // ray against the surface we just hit and `schlick()` as a propability.
        let scattered_dir: Float3;

        // Can we refract?
        if let Some(refracted) = ray_in.dir.refract(outward_normal,
                                                    refraction_index)
        {
            // Yes, and we usually will if we can.
            // But first, we check a random number against the `schlick`
            // function. This represents the odds of *reflecting* instead.
            if random_float() >= schlick(cosine, refraction_index) {
                scattered_dir = refracted;
            } else {
                // Probability test failed: just reflect.
                scattered_dir = reflected;
            }
        } else {
            // We can't refract: just reflect.
            scattered_dir = reflected;
        }

        *scattered = Ray {
            origin: record.p,
            dir:    scattered_dir,
            t:      ray_in.t,
        };
        true
    }
}
