//! Shapes in a scene.

mod triangle;

pub use triangle::Triangle;

use crate::aabb::Aabb;
use crate::path::Path;
use crate::scene::SceneSpace;
use id_arena::Id;
use std::fmt::Debug;

/// An identifier for a shape that has been added to a `Scene`.
pub type ShapeId = Id<Box<Shape<i64, SceneSpace>>>;

/// Any kind of shape!
pub trait Shape<T, U>: Debug {
    /// Get the paths that will render this shape and/or fill it with a texture.
    fn paths(&self) -> Vec<Path<T, U>>;

    /// Get this shape's axis-aligned bounding box.
    fn aabb(&self) -> Aabb<T, U>;
}
