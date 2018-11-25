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

pub fn factors(num: u32) -> impl Iterator<Item=u32> {
    struct FactorIter {
        max: u32,
        cur: u32,
    }

    impl Iterator for FactorIter {
        type Item = u32;
        fn next(&mut self) -> Option<Self::Item> {
            if self.cur >= self.max {
                return None;
            }
            if self.cur == 0 {
                self.cur = 1;
                return Some(1); // that I use to know
            }

            loop {
                self.cur += 1;
                if self.max % self.cur == 0 {
                    break;
                }
                debug_assert!(self.cur < self.max);
            }

            Some(self.cur)
        }
    }

    FactorIter {
        max: num,
        cur: 0,
    }
}

#[cfg(test)]
mod t {
    #[test]
    fn check_factors() {
        let known_factors: [ &[u32]; 33 ] = [
            /*  0 => */ &[],
            /*  1 => */ &[1],
            /*  2 => */ &[1, 2],
            /*  3 => */ &[1, 3],
            /*  4 => */ &[1, 2, 4],
            /*  5 => */ &[1, 5],
            /*  6 => */ &[1, 2, 3, 6],
            /*  7 => */ &[1, 7],
            /*  8 => */ &[1, 2, 4, 8],
            /*  9 => */ &[1, 3, 9],
            /* 10 => */ &[1, 2, 5, 10],
            /* 11 => */ &[1, 11],
            /* 12 => */ &[1, 2, 3, 4, 6, 12],
            /* 13 => */ &[1, 13],
            /* 14 => */ &[1, 2, 7, 14],
            /* 15 => */ &[1, 3, 5, 15],
            /* 16 => */ &[1, 2, 4, 8, 16],
            /* 17 => */ &[1, 17],
            /* 18 => */ &[1, 2, 3, 6, 9, 18],
            /* 19 => */ &[1, 19],
            /* 20 => */ &[1, 2, 4, 5, 10, 20],
            /* 21 => */ &[1, 3, 7, 21],
            /* 22 => */ &[1, 2, 11, 22],
            /* 23 => */ &[1, 23],
            /* 24 => */ &[1, 2, 3, 4, 6, 8, 12, 24],
            /* 25 => */ &[1, 5, 25],
            /* 26 => */ &[1, 2, 13, 26],
            /* 27 => */ &[1, 3, 9, 27],
            /* 28 => */ &[1, 2, 4, 7, 14, 28],
            /* 29 => */ &[1, 29],
            /* 30 => */ &[1, 2, 3, 5, 6, 10, 15, 30],
            /* 31 => */ &[1, 31],
            /* 32 => */ &[1, 2, 4, 8, 16, 32],
        ];
        for (num, known) in known_factors.iter().cloned().enumerate() {
            let ours: Vec<_> = crate::math::factors(num as u32).collect();
            assert_eq!(ours, known);
        }
    }
}
