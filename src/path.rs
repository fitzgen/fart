//! Paths describe a series lines and curves.
//!
//! Everything in a scene is eventually lowered into a series of paths, which
//! are finally compiled to SVG.

use crate::{Point2, Vector2};

/// A series of line commands that describe a path.
#[derive(Clone, Debug, Default)]
pub struct Path {
    /// This path's line commands.
    pub commands: Vec<LineCommand>,
}

/// An individual line command segment within a `Path`.
#[derive(Clone, Debug)]
pub enum LineCommand {
    /// Move the cursor to the given point.
    MoveTo(Point2),
    /// Move the cursor relative from its current position by the given vector.
    MoveBy(Vector2),

    /// Draw a line to the given point.
    LineTo(Point2),
    /// Draw a line relative from its current position by the given vector.
    LineBy(Vector2),

    /// Draw a horizontal line to the given x coordinate.
    HorizontalLineTo(f64),
    /// Draw a horizontal line relative from its current position by the given x
    /// delta.
    HorizontalLineBy(f64),

    /// Draw a vertical line to the given y coordinate.
    VerticalLineTo(f64),
    /// Draw a vertical line relative from its current position by the given y
    /// delta.
    VerticalLineBy(f64),

    /// Draw a straight line from the current position back to the first
    /// position in the path.
    Close,

    /// Draw a cubic bezier curve from the current position with the given
    /// absolute points.
    CubicBezierTo {
        /// The first control point, describing the curve out from the starting
        /// position.
        control_1: Point2,
        /// The second control point, describing the curve into the ending
        /// position.
        control_2: Point2,
        /// The ending position.
        end: Point2,
    },
    /// Draw a cubic bezier curve from the current position with the give
    /// relative vectors.
    CubicBezierBy {
        /// The first control point, describing the curve out from the starting
        /// position.
        control_1: Vector2,
        /// The second control point, describing the curve into the ending
        /// position.
        control_2: Vector2,
        /// The ending position.
        end: Vector2,
    },

    /// Draw a smooth cubic bezier curve from the current position with the
    /// given absolute points.
    SmoothCubicBezierTo {
        /// The control point describing the curve.
        control: Point2,
        /// The ending position.
        end: Point2,
    },
    /// Draw a smooth cubic bezier curve from the current position with the
    /// given relative vectors.
    SmoothCubicBezierBy {
        /// The control point describing the curve.
        control: Vector2,
        /// The ending position.
        end: Vector2,
    },

    /// Draw a quadratic bezier curve from the current position with the given
    /// absolute points.
    QuadraticBezierTo {
        /// The control point describing the curve.
        control: Point2,
        /// The ending position.
        end: Point2,
    },
    /// Draw a quadratic bezier curve from the current position with the given
    /// relative vectors.
    QuadraticBezierBy {
        /// The control point describing the curve.
        control: Vector2,
        /// The ending position.
        end: Vector2,
    },

    /// Draw a smooth quadratic bezier curve from the current position to the
    /// given absolute point.
    SmoothQuadtraticCurveTo(Point2),
    /// Draw a smooth quadratic bezier curve from the current position to the
    /// given relative vector.
    SmoothQuadtraticCurveBy(Vector2),

    /// Draw an arc to the given absolute end point.
    ArcTo {
        /// The x radius of the arc's ellipse.
        x_radius: f64,
        /// The y radius of the arc's ellipse.
        y_radius: f64,
        /// The rotation of the arc.
        x_axis_rotation: euclid::Angle<f64>,
        /// Should the arc be greater than or less than 180 degrees?
        large_arc_flag: bool,
        /// Determines whether to begin moving at positive or negative angles.
        sweep_flag: bool,
        /// The ending position of the arc.
        end: Point2,
    },
    /// Draw an arc with the given relative end vector.
    ArcBy {
        /// The x radius of the arc's ellipse.
        x_radius: f64,
        /// The y radius of the arc's ellipse.
        y_radius: f64,
        /// The rotation of the arc.
        x_axis_rotation: f64,
        /// Should the arc be greater than or less than 180 degrees?
        large_arc_flag: bool,
        /// Determines whether to begin moving at positive or negative angles.
        sweep_flag: bool,
        /// The ending position of the arc.
        end: Vector2,
    },
}

impl Path {
    /// Construct a new, empty path.
    pub fn new() -> Path {
        Path::default()
    }

    /// Construct a new path with the given line commands.
    pub fn with_commands<I>(commands: I) -> Path
    where
        I: IntoIterator<Item = LineCommand>,
    {
        Path {
            commands: commands.into_iter().collect(),
        }
    }
}

impl<'a> From<&'a Path> for svg::node::element::Path {
    fn from(path: &'a Path) -> svg::node::element::Path {
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
            .set("stroke-width", "3")
            .set("stroke", "black")
            .set("fill", "none")
            .set("d", data)
    }
}
