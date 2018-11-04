
use std::{
    convert::Into,
    mem,
    ops,
};

pub type Float = f64;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Float3 {
    x: Float,
    y: Float,
    z: Float,
}

impl Float3 {
    pub fn new() -> Float3 {
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

impl ops::SubAssign<Float3> for Float3 {
    fn sub_assign(&mut self, rhs: Float3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
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

        // Float3 /= $prim
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

    use crate::geometry::{
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

}
