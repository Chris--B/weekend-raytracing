
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

impl ops::Add for Float3 {
    type Output = Self;
    fn add(self, rhs: Float3) -> Float3 {
        Float3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
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

impl ops::Mul<Float> for Float3 {
    type Output = Self;
    fn mul(self, rhs: Float) -> Float3 {
        Float3 {
            x: rhs * self.x,
            y: rhs * self.y,
            z: rhs * self.z,
        }
    }
}

impl ops::Div<Float> for Float3 {
    type Output = Self;
    fn div(self, rhs: Float) -> Float3 {
        Float3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}


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


}