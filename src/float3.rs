
use std::{
    convert::Into,
    mem,
    ops,
};

use rand::prelude::*;

pub type Float = f64;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Float3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

impl Float3 {

    // ---- Constructors ----------

    pub const fn new() -> Float3 {
        Float3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn xyz<F: Into<Float>>(x: F, y: F, z: F) -> Float3 {
        Float3 {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    pub fn xy<F: Into<Float>>(x: F, y: F) -> Float3 {
        Float3 {
            x: x.into(),
            y: y.into(),
            z: 0.0,
        }
    }

    pub fn xxx<F: Into<Float>>(x: F) -> Float3 {
        let x = x.into();
        Float3 {
            x: x,
            y: x,
            z: x,
        }
    }

    /// Returns a random point uniformly from the unit sphere,
    /// centered at the origin.
    pub fn random_in_sphere() -> Float3 {
        // This is a bad way to do this. With our 200x100 image, we reliably
        // run this loop 18 times without finding a point.
        // ಠ_ಠ
        loop {
            let x: Float = 2.0 * random::<Float>() - 1.0;
            let y: Float = 2.0 * random::<Float>() - 1.0;
            let z: Float = 2.0 * random::<Float>() - 1.0;
            let p = Float3 { x, y, z };
            if p.length_sq() < 1.0 {
                return p;
            }
        }
    }

    // ---- Access/Translations ----------

    pub fn as_slice(&self) -> &[Float; 3] {
        unsafe {
            mem::transmute(&self.x)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [Float; 3] {
        unsafe {
            mem::transmute(&mut self.x)
        }
    }

    // ---- Mathy Operations ----------

    /// Reflects the vector about a surface with normal `n`.
    pub fn reflect(&self, n: Float3) -> Float3 {
        *self - 2.0 * self.dot(&n) * n
    }

    pub fn sqrt(&self) -> Float3 {
        Float3 {
            x: self.x.sqrt(),
            y: self.y.sqrt(),
            z: self.z.sqrt(),
        }
    }

    pub fn lerp(t: Float, a: Float3, b: Float3) -> Float3 {
        (1.0 - t) * a + t * b
    }

    pub fn dot(&self, other: &Float3) -> Float {
        (self.x * other.x) +
        (self.y * other.y) +
        (self.z * other.z)
    }

    pub fn cross(&self, other: &Float3) -> Float3 {
        let v1 = self;
        let v2 = &other;
        Float3 {
            x:  (v1.y*v2.z - v1.z*v2.y),
            y: -(v1.x*v2.z - v1.z*v2.x),
            z:  (v1.x*v2.y - v1.y*v2.x),
        }
    }

    pub fn length(&self) -> Float {
        self.length_sq().sqrt()
    }

    pub fn length_sq(&self) -> Float {
        self.dot(self)
    }

    pub fn unit(&self) -> Float3 {
        *self / self.length()
    }

    pub fn make_unit(&mut self) {
        *self /= self.length()
    }
}

impl ops::Add<Float3> for Float3 {
    type Output = Self;
    fn add(self, rhs: Float3) -> Float3 {
        Float3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::AddAssign<Float3> for Float3 {
    fn add_assign(&mut self, rhs: Float3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Sub for Float3 {
    type Output = Self;
    fn sub(self, rhs: Float3) -> Float3 {
        Float3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::SubAssign<Float3> for Float3 {
    fn sub_assign(&mut self, rhs: Float3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl ops::Neg for Float3 {
    type Output = Float3;
    fn neg(self) -> Float3 {
        Float3 {
            x: -self.x,
            y: -self.y,
            z: -self.z
        }
    }
}

macro_rules! impl_scalar_add_for {
    ($prim:ty) => {
        // $prim + Float3
        impl ops::Add<$prim> for Float3 {
            type Output = Float3;
            fn add(self, rhs: $prim) -> Float3 {
                Float3 {
                    x: self.x + rhs as Float,
                    y: self.y + rhs as Float,
                    z: self.z + rhs as Float,
                }
            }
        }

        // Float3 + $prim
        impl ops::Add<Float3> for $prim {
            type Output = Float3;
            fn add(self, rhs: Float3) -> Float3 {
                Float3 {
                    x: self as Float + rhs.x,
                    y: self as Float + rhs.y,
                    z: self as Float + rhs.z,
                }
            }
        }

        // Float3 += $prim
        impl ops::AddAssign<$prim> for Float3 {
            fn add_assign(&mut self, rhs: $prim) {
                self.x += rhs as Float;
                self.y += rhs as Float;
                self.z += rhs as Float;
            }
        }
    }
}

macro_rules! impl_scalar_mul_for {
    ($prim:ty) => {
        // $prim * Float3
        impl ops::Mul<$prim> for Float3 {
            type Output = Float3;
            fn mul(self, rhs: $prim) -> Float3 {
                Float3 {
                    x: self.x * rhs as Float,
                    y: self.y * rhs as Float,
                    z: self.z * rhs as Float,
                }
            }
        }

        // Float3 * $prim
        impl ops::Mul<Float3> for $prim {
            type Output = Float3;
            fn mul(self, rhs: Float3) -> Float3 {
                Float3 {
                    x: self as Float * rhs.x,
                    y: self as Float * rhs.y,
                    z: self as Float * rhs.z,
                }
            }
        }

        // Float3 *= $prim
        impl ops::MulAssign<$prim> for Float3 {
            fn mul_assign(&mut self, rhs: $prim) {
                self.x *= rhs as Float;
                self.y *= rhs as Float;
                self.z *= rhs as Float;
            }
        }
    }
}

macro_rules! impl_scalar_div_for {
    ($prim:ty) => {
        // $prim / Float3
        impl ops::Div<$prim> for Float3 {
            type Output = Float3;
            fn div(self, rhs: $prim) -> Float3 {
                Float3 {
                    x: self.x / (rhs as Float),
                    y: self.y / (rhs as Float),
                    z: self.z / (rhs as Float),
                }
            }
        }

        // Float3 / $prim
        impl ops::Div<Float3> for $prim {
            type Output = Float3;
            fn div(self, rhs: Float3) -> Float3 {
                Float3 {
                    x: self as Float / rhs.x,
                    y: self as Float / rhs.y,
                    z: self as Float / rhs.z,
                }
            }
        }

        // Float3 /= $prim
        impl ops::DivAssign<$prim> for Float3 {
            fn div_assign(&mut self, rhs: $prim) {
                self.x /= rhs as Float;
                self.y /= rhs as Float;
                self.z /= rhs as Float;
            }
        }
    }
}

// Scalar '+' overloads
impl_scalar_add_for!(f32);
impl_scalar_add_for!(f64);

impl_scalar_add_for!(u8);
impl_scalar_add_for!(u16);
impl_scalar_add_for!(u32);
impl_scalar_add_for!(u64);

impl_scalar_add_for!(i8);
impl_scalar_add_for!(i16);
impl_scalar_add_for!(i32);
impl_scalar_add_for!(i64);

impl_scalar_add_for!(usize);
impl_scalar_add_for!(isize);

// Scalar '*' overloads
impl_scalar_mul_for!(f32);
impl_scalar_mul_for!(f64);

impl_scalar_mul_for!(u8);
impl_scalar_mul_for!(u16);
impl_scalar_mul_for!(u32);
impl_scalar_mul_for!(u64);

impl_scalar_mul_for!(i8);
impl_scalar_mul_for!(i16);
impl_scalar_mul_for!(i32);
impl_scalar_mul_for!(i64);

impl_scalar_mul_for!(usize);
impl_scalar_mul_for!(isize);

// Scalar '/' overloads
impl_scalar_div_for!(f32);
impl_scalar_div_for!(f64);

impl_scalar_div_for!(u8);
impl_scalar_div_for!(u16);
impl_scalar_div_for!(u32);
impl_scalar_div_for!(u64);

impl_scalar_div_for!(i8);
impl_scalar_div_for!(i16);
impl_scalar_div_for!(i32);
impl_scalar_div_for!(i64);

impl_scalar_div_for!(usize);
impl_scalar_div_for!(isize);

#[cfg(test)]
mod t {
    use std::mem;

    use crate::float3::{
        Float,
        Float3,
    };

    #[test]
    fn check_as_slice() {
        let a = Float3::xyz(1, 2, 3);

        let slice: &[Float; 3] = a.as_slice();
        // We used mem::transmute on a *reference*,
        // so this isn't statically checked for us.
        assert_eq!(mem::size_of_val(&*slice), mem::size_of::<Float3>());
        assert_eq!(slice, &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn check_as_mut_slice() {
        let mut a = Float3::xyz(1, 2, 3);

        let slice: &mut [Float; 3] = a.as_mut_slice();
        // We used mem::transmute on a *reference*,
        // so this isn't statically checked for us.
        assert_eq!(mem::size_of_val(&*slice), mem::size_of::<Float3>());
        assert_eq!(slice, &[1.0, 2.0, 3.0]);

        // Writing through this slice should be visible through 'a',
        // and definitely shouldn't crash.
        slice[0] = -1.0;
        slice[1] = -2.0;
        slice[2] = -3.0;
        assert_eq!(a, Float3::xyz(-1, -2, -3));
    }

    #[test]
    fn check_arithmatic() {
        let mut a = Float3::xyz(1, 2, 3);

        // Scalar Mul and Div
        a = 5 * a;
        assert_eq!(a, Float3::xyz(5, 10, 15));
        a = a * 5u8;
        assert_eq!(a, Float3::xyz(25, 50, 75));
        a = a / 5isize;
        assert_eq!(a, Float3::xyz(5, 10, 15));
        let unused: Float3 = 5 / a;
        assert_eq!(unused, Float3::xyz(5.0 / 5.0, 5.0 / 10.0, 5.0 / 15.0));

        // Scalar Mul/Div Assign
        a *= 2i16;
        assert_eq!(a, Float3::xyz(10, 20, 30));
        a /= 10u16;
        assert_eq!(a, Float3::xyz(1.0, 2.0, 3.0));
        // Note: `a` has now returned to its original value.

        // Vector Add
        assert_eq!(a + Float3::xxx(10), Float3::xyz(11, 12, 13));
        a += Float3::xxx(10);
        assert_eq!(a, Float3::xyz(11, 12, 13));

        // Vector Sub
        assert_eq!(a - Float3::xxx(10), Float3::xyz(1, 2, 3));
        a -= Float3::xxx(10);
        assert_eq!(a, Float3::xyz(1, 2, 3));
    }

    #[test]
    fn check_mathy() {
        let i = Float3::xyz(1, 0, 0);
        let j = Float3::xyz(0, 1, 0);
        let k = Float3::xyz(0, 0, 1);

        // The three axes "cross" in a loop: ijk, jki, kij, etc.
        // The cross of the first two always equals the third.
        assert_eq!(i.cross(&j), k);
        assert_eq!(j.cross(&k), i);
        assert_eq!(k.cross(&i), j);

        // If you "cross" the loop backwards, the results' signs flip.
        assert_eq!(j.cross(&i), -k);
        assert_eq!(k.cross(&j), -i);
        assert_eq!(i.cross(&k), -j);

        // Just for good measure, here's an example from "Paul's Notes":
        let a = Float3::xyz(2, 1, -1);
        let b = Float3::xyz(-3, 4, 1);

        // Same extra sanity checks
        // Anything crossed with itself is zero.
        assert_eq!(a.cross(&a), Float3::xxx(0.0));
        assert_eq!(b.cross(&b), Float3::xxx(0.0));

        // Solutions from Paul's Notes.
        assert_eq!(a.cross(&b), Float3::xyz(5, 1, 11));
        assert_eq!(b.cross(&a), Float3::xyz(-5, -1, -11));
    }
}
