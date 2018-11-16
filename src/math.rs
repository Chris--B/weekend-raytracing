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
pub fn random_in_sphere() -> Float3 {
    // This is a bad way to do this. With our 200x100 image, we reliably
    // run this loop 18 times without finding a point.
    // ಠ_ಠ
    loop {
        let x: Float = random_sfloat();
        let y: Float = random_sfloat();
        let z: Float = random_sfloat();
        let p = Float3 { x, y, z };
        if p.length_sq() < 1.0 {
            return p;
        }
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

pub fn random_float_in(start: Float, end: Float) -> Float {
    (end - start) * rand::random::<Float>() + end
}

pub fn random_sfloat() -> Float {
    2.0 * rand::random::<Float>() - 1.0
}
