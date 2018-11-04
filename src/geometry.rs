
use std::ops;
use std::convert::Into;

type Float = f64;

#[derive(Copy, Clone, Debug)]
pub struct Float3 {
    x: Float,
    y: Float,
    z: Float
}

impl Float3 {
    pub fn new<F: Into<Float>>(x: F, y: F, z: F) -> Float3 {
        Float3 {
            x: x.into(),
            y: y.into(),
            z: z.into(),
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
    use super::*;

    #[test]
    fn check_add() {
        let v = Vec3::new(1, 2, 3);
    }
}