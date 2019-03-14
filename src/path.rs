//! Paths describe a series lines and curves.
//!
//! Everything in a scene is eventually lowered into a series of paths, which
//! are finally compiled to SVG.

use crate::scene::SceneSpace;
use euclid::{TypedPoint2D, TypedVector2D};
use std::fmt::Debug;

/// A series of line commands that describe a path.
#[derive(Clone, Debug, Default)]
pub struct Path<T, U> {
    /// This path's line commands.
    pub commands: Vec<LineCommand<T, U>>,
}

/// An individual line command segment within a `Path`.
#[derive(Clone, Debug)]
pub enum LineCommand<T, U> {
    /// Move the cursor to the given point.
    MoveTo(TypedPoint2D<T, U>),
    /// Move the cursor relative from its current position by the given vector.
    MoveBy(TypedVector2D<T, U>),

    /// Draw a line to the given point.
    LineTo(TypedPoint2D<T, U>),
    /// Draw a line relative from its current position by the given vector.
    LineBy(TypedVector2D<T, U>),

    /// Draw a horizontal line to the given x coordinate.
    HorizontalLineTo(T),
    /// Draw a horizontal line relative from its current position by the given x
    /// delta.
    HorizontalLineBy(T),

    /// Draw a vertical line to the given y coordinate.
    VerticalLineTo(T),
    /// Draw a vertical line relative from its current position by the given y
    /// delta.
    VerticalLineBy(T),

    /// Draw a straight line from the current position back to the first
    /// position in the path.
    Close,

    /// Draw a cubic bezier curve from the current position with the given
    /// absolute points.
    CubicBezierTo {
        /// The first control point, describing the curve out from the starting
        /// position.
        control_1: TypedPoint2D<T, U>,
        /// The second control point, describing the curve into the ending
        /// position.
        control_2: TypedPoint2D<T, U>,
        /// The ending position.
        end: TypedPoint2D<T, U>,
    },
    /// Draw a cubic bezier curve from the current position with the give
    /// relative vectors.
    CubicBezierBy {
        /// The first control point, describing the curve out from the starting
        /// position.
        control_1: TypedVector2D<T, U>,
        /// The second control point, describing the curve into the ending
        /// position.
        control_2: TypedVector2D<T, U>,
        /// The ending position.
        end: TypedVector2D<T, U>,
    },

    /// Draw a smooth cubic bezier curve from the current position with the
    /// given absolute points.
    SmoothCubicBezierTo {
        /// The control point describing the curve.
        control: TypedPoint2D<T, U>,
        /// The ending position.
        end: TypedPoint2D<T, U>,
    },
    /// Draw a smooth cubic bezier curve from the current position with the
    /// given relative vectors.
    SmoothCubicBezierBy {
        /// The control point describing the curve.
        control: TypedVector2D<T, U>,
        /// The ending position.
        end: TypedVector2D<T, U>,
    },

    /// Draw a quadratic bezier curve from the current position with the given
    /// absolute points.
    QuadraticBezierTo {
        /// The control point describing the curve.
        control: TypedPoint2D<T, U>,
        /// The ending position.
        end: TypedPoint2D<T, U>,
    },
    /// Draw a quadratic bezier curve from the current position with the given
    /// relative vectors.
    QuadraticBezierBy {
        /// The control point describing the curve.
        control: TypedVector2D<T, U>,
        /// The ending position.
        end: TypedVector2D<T, U>,
    },

    /// Draw a smooth quadratic bezier curve from the current position to the
    /// given absolute point.
    SmoothQuadtraticCurveTo(TypedPoint2D<T, U>),
    /// Draw a smooth quadratic bezier curve from the current position to the
    /// given relative vector.
    SmoothQuadtraticCurveBy(TypedVector2D<T, U>),

    /// Draw an arc to the given absolute end point.
    ArcTo {
        /// The x radius of the arc's ellipse.
        x_radius: T,
        /// The y radius of the arc's ellipse.
        y_radius: T,
        /// The rotation of the arc.
        x_axis_rotation: euclid::Angle<T>,
        /// Should the arc be greater than or less than 180 degrees?
        large_arc_flag: bool,
        /// Determines whether to begin moving at positive or negative angles.
        sweep_flag: bool,
        /// The ending position of the arc.
        end: TypedPoint2D<T, U>,
    },
    /// Draw an arc with the given relative end vector.
    ArcBy {
        /// The x radius of the arc's ellipse.
        x_radius: T,
        /// The y radius of the arc's ellipse.
        y_radius: T,
        /// The rotation of the arc.
        x_axis_rotation: T,
        /// Should the arc be greater than or less than 180 degrees?
        large_arc_flag: bool,
        /// Determines whether to begin moving at positive or negative angles.
        sweep_flag: bool,
        /// The ending position of the arc.
        end: TypedVector2D<T, U>,
    },
}

impl<T, U> Path<T, U> {
    /// Construct a new, empty path.
    pub fn new() -> Path<T, U> {
        Path { commands: vec![] }
    }

    /// Construct a new path with the given line commands.
    pub fn with_commands<I>(commands: I) -> Path<T, U>
    where
        I: IntoIterator<Item = LineCommand<T, U>>,
    {
        Path {
            commands: commands.into_iter().collect(),
        }
    }
}

impl<'a, T> From<&'a Path<T, SceneSpace>> for svg::node::element::Path
where
    T: Copy + Debug + Into<svg::node::element::path::Parameters>,
{
    fn from(path: &'a Path<T, SceneSpace>) -> svg::node::element::Path {
        let mut data = svg::node::element::path::Data::new();
        for cmd in &path.commands {
            data = match cmd {
                LineCommand::MoveTo(p) => data.move_to((p.x, p.y)),
                LineCommand::MoveBy(v) => data.move_by((v.x, v.y)),
                LineCommand::LineTo(p) => data.line_to((p.x, p.y)),
                LineCommand::LineBy(v) => data.line_by((v.x, v.y)),
                LineCommand::HorizontalLineTo(x) => data.horizontal_line_to(*x),
                LineCommand::HorizontalLineBy(dx) => data.horizontal_line_by(*dx),
                LineCommand::VerticalLineTo(y) => data.vertical_line_to(*y),
                LineCommand::VerticalLineBy(dy) => data.vertical_line_by(*dy),
                LineCommand::Close => data.close(),
                cmd => unimplemented!("Have not implemented support for command yet: {:?}", cmd),
            };
        }
        svg::node::element::Path::new()
            .set("stroke", "black")
            .set("fill", "none")
            .set("d", data)
    }
}
