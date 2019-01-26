//! Shapes in a scene.

mod triangle;

pub use triangle::Triangle;

use crate::aabb::AxisAlignedBoundingBox;
use crate::path::Path;
use id_arena::Id;
use std::fmt::Debug;

/// An identifier for a shape that has been added to a `Scene`.
pub type ShapeId = Id<Box<Shape>>;

/// Any kind of shape!
pub trait Shape: Debug {
    /// Get the paths that will rener this shape and/or fill it with a texture.
    fn paths(&self) -> Vec<Path>;

    /// Get this shape's axis-aligned bounding box.
    fn aabb(&self) -> AxisAlignedBoundingBox;
}
