//! The most common functionality re-exported.

pub use crate::{
    clamp, map_range,
    path::{LineCommand, Path},
    scene::{Inches, Scene, SceneSpace},
    shape::{Shape, ShapeId, Triangle},
};
pub use euclid::{point2, vec2};
pub use euclid_aabb::Aabb;
pub use rand::{
    distributions::{Distribution, Normal, Uniform},
    Rng,
};
