use std::f64::consts;
use crate::prelude::*;

/// When you look at a window at a steep angle, it becomes a mirror.
/// This is a simple approximation to that by Christophe Schlick.
pub fn schlick(cosine: Float, refraction_index: Float) -> Float {
    let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
    r0 *= r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

/// Returns a random point uniformly from the unit sphere,
/// centered at the origin.
/// See for details: https://math.stackexchange.com/a/822863
pub fn random_in_sphere() -> Float3 {
    let λ = random_float();
    let u = random_sfloat();
    let φ =  2.0 * consts::PI * random_float();

    Float3 {
        x: λ.powf(3.0) * (1.0 - u* u).sqrt() * φ.cos(),
        y: λ.powf(3.0) * (1.0 - u* u).sqrt() * φ.sin(),
        z: λ.powf(3.0) * u,
    }
}

/// Returns a random point uniformly from the unit disk.
/// Disks are 2D, so the Z component is always zero.
pub fn random_in_disk() -> Float3 {
    // Oh good, more of this.
    loop {
        let x = random_sfloat();
        let y = random_sfloat();
        let p = Float3::xyz(x, y, 0.0);
        if p.length_sq() < 1.0 {
            return p;
        }
    }
}

pub fn random_float() -> Float {
    rand::random()
}

pub fn random_sfloat() -> Float {
    2.0 * rand::random::<Float>() - 1.0
}
