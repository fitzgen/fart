//! Shapes in a scene.

mod triangle;

pub use triangle::Triangle;

use crate::aabb::Aabb;
use crate::path::Path;
use crate::scene::SceneSpace;
use id_arena::Id;
use std::borrow::Cow;
use std::fmt::Debug;

/// An identifier for a shape that has been added to a `Scene`.
pub type ShapeId = Id<Box<dyn Shape<i64, SceneSpace>>>;

/// Any kind of shape!
pub trait Shape<T, U>: Debug {
    /// Get the paths that will render this shape and/or fill it with a texture.
    fn paths(&self) -> Vec<Path<T, U>>;

    /// Get this shape's axis-aligned bounding box.
    fn aabb(&self) -> Aabb<T, U>;
}

/// A `Shape` wrapper type that forces all of the inner shape's paths to be of a
/// certain color. Created using `ShapeExt::color`.
#[derive(Debug, Clone)]
pub struct Color<S> {
    inner: S,
    color: Cow<'static, str>,
}

impl<S, T, U> Shape<T, U> for Color<S>
where
    S: Shape<T, U>,
{
    fn paths(&self) -> Vec<Path<T, U>> {
        let mut paths = self.inner.paths();
        for p in &mut paths {
            p.color = self.color.clone().into();
        }
        paths
    }

    fn aabb(&self) -> Aabb<T, U> {
        self.inner.aabb()
    }
}

/// An extension trait for shapes to add various helper utilities.
pub trait ShapeExt<T, U>: Shape<T, U> {
    /// Force this shape's paths to be of the given color.
    fn color<C>(self, color: C) -> Color<Self>
    where
        C: Into<Cow<'static, str>>,
        Self: Sized,
    {
        Color {
            inner: self,
            color: color.into(),
        }
    }
}

impl<S, T, U> ShapeExt<T, U> for S where S: Shape<T, U> {}
