use super::Shape;
use crate::aabb::Aabb;
use crate::path::{LineCommand, Path};
use euclid::TypedPoint2D;
use num_traits::Num;
use std::fmt::Debug;

/// A plain triangle.
///
/// Does not contain any sort of fill or texture.
#[derive(Clone, Debug, PartialEq)]
pub struct Triangle<T, U> {
    /// The first point.
    pub a: TypedPoint2D<T, U>,
    /// The second point.
    pub b: TypedPoint2D<T, U>,
    /// The third point.
    pub c: TypedPoint2D<T, U>,
}

impl<T, U> Shape<T, U> for Triangle<T, U>
where
    T: Num + Copy + Debug + PartialOrd,
    U: Copy + Debug,
{
    fn paths(&self) -> Vec<Path<T, U>> {
        vec![Path::with_commands(vec![
            LineCommand::MoveTo(self.a),
            LineCommand::LineTo(self.b),
            LineCommand::LineTo(self.c),
            LineCommand::Close,
        ])]
    }

    fn aabb(&self) -> Aabb<T, U> {
        Aabb::for_vertices([self.a, self.b, self.c].iter().cloned())
    }
}
