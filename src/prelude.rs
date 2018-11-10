//! The preldue includes commonly-accessed items from the submodules.
//! If it's only used in main.rs, it does not belong here.

pub use crate::float3::{
    Float,
    Float3,
};
pub use crate::hitable::HitRecord;
pub use crate::material::Material;
pub use crate::math::*;
pub use crate::ray::Ray;