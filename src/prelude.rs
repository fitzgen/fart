//! The most common functionality re-exported.

pub use crate::{
    path::{LineCommand, Path},
    scene::{Inches, Scene, SceneSpace},
    shape::{Shape, ShapeId, Triangle},
};
pub use euclid::{point2, vec2};
pub use fart_aabb::Aabb;
pub use fart_utils::{clamp, map_range};
pub use rand::{
    distributions::{Distribution, Normal, Uniform},
    Rng,
};
