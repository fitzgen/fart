use super::Shape;
use crate::aabb::AxisAlignedBoundingBox;
use crate::path::{LineCommand, Path};
use crate::Point2;

/// A plain triangle.
///
/// Does not contain any sort of fill or texture.
#[derive(Clone, Debug, PartialEq)]
pub struct Triangle {
    /// The first point.
    pub a: Point2,
    /// The second point.
    pub b: Point2,
    /// The third point.
    pub c: Point2,
}

impl Shape for Triangle {
    fn paths(&self) -> Vec<Path> {
        vec![Path::with_commands(vec![
            LineCommand::MoveTo(self.a),
            LineCommand::LineTo(self.b),
            LineCommand::LineTo(self.c),
            LineCommand::Close,
        ])]
    }

    fn aabb(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox::for_vertices([self.a, self.b, self.c].iter().cloned())
    }
}
