//! Paths describe a series lines and curves.
//!
//! Everything in a scene is eventually lowered into a series of paths, which
//! are finally compiled to SVG.

use crate::canvas::CanvasSpace;
use euclid::{point2, vec2, Point2D, Vector2D};
use num_traits::{Num, NumAssign, NumCast, Signed};
use std::borrow::Cow;
use std::fmt::Debug;
use std::iter;
use std::marker::PhantomData;

/// A series of line commands that describe a path.
#[derive(Clone, Debug)]
pub struct Path<T, U> {
    /// This path's color.
    pub color: String,

    /// This path's line commands.
    pub commands: Vec<LineCommand<T, U>>,
}

impl<T, U> Default for Path<T, U> {
    fn default() -> Path<T, U> {
        Path {
            color: "black".into(),
            commands: vec![],
        }
    }
}

impl<T, U> ToPaths<T, U> for Path<T, U>
where
    Self: Clone,
{
    type Paths = iter::Once<Self>;

    fn to_paths(&self) -> Self::Paths {
        iter::once(self.clone())
    }
}

/// An individual line command segment within a `Path`.
#[derive(Clone, Debug)]
pub enum LineCommand<T, U> {
    /// Move the cursor to the given point.
    MoveTo(Point2D<T, U>),
    /// Move the cursor relative from its current position by the given vector.
    MoveBy(Vector2D<T, U>),

    /// Draw a line to the given point.
    LineTo(Point2D<T, U>),
    /// Draw a line relative from its current position by the given vector.
    LineBy(Vector2D<T, U>),

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
        control_1: Point2D<T, U>,
        /// The second control point, describing the curve into the ending
        /// position.
        control_2: Point2D<T, U>,
        /// The ending position.
        end: Point2D<T, U>,
    },
    /// Draw a cubic bezier curve from the current position with the give
    /// relative vectors.
    CubicBezierBy {
        /// The first control point, describing the curve out from the starting
        /// position.
        control_1: Vector2D<T, U>,
        /// The second control point, describing the curve into the ending
        /// position.
        control_2: Vector2D<T, U>,
        /// The ending position.
        end: Vector2D<T, U>,
    },

    /// Draw a smooth cubic bezier curve from the current position with the
    /// given absolute points.
    SmoothCubicBezierTo {
        /// The control point describing the curve.
        control: Point2D<T, U>,
        /// The ending position.
        end: Point2D<T, U>,
    },
    /// Draw a smooth cubic bezier curve from the current position with the
    /// given relative vectors.
    SmoothCubicBezierBy {
        /// The control point describing the curve.
        control: Vector2D<T, U>,
        /// The ending position.
        end: Vector2D<T, U>,
    },

    /// Draw a quadratic bezier curve from the current position with the given
    /// absolute points.
    QuadraticBezierTo {
        /// The control point describing the curve.
        control: Point2D<T, U>,
        /// The ending position.
        end: Point2D<T, U>,
    },
    /// Draw a quadratic bezier curve from the current position with the given
    /// relative vectors.
    QuadraticBezierBy {
        /// The control point describing the curve.
        control: Vector2D<T, U>,
        /// The ending position.
        end: Vector2D<T, U>,
    },

    /// Draw a smooth quadratic bezier curve from the current position to the
    /// given absolute point.
    SmoothQuadtraticCurveTo(Point2D<T, U>),
    /// Draw a smooth quadratic bezier curve from the current position to the
    /// given relative vector.
    SmoothQuadtraticCurveBy(Vector2D<T, U>),

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
        end: Point2D<T, U>,
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
        end: Vector2D<T, U>,
    },
}

impl<T, U> LineCommand<T, U>
where
    T: Copy + Num + PartialOrd,
{
    /// Transform this line command with the given linear transformation and
    /// return the new, transformed line command.
    pub fn transform<V>(&self, transformation: &euclid::Transform2D<T, U, V>) -> LineCommand<T, V> {
        match *self {
            LineCommand::MoveTo(p) => LineCommand::MoveTo(transformation.transform_point(p)),
            LineCommand::MoveBy(v) => LineCommand::MoveBy(transformation.transform_vector(v)),
            LineCommand::LineTo(p) => LineCommand::LineTo(transformation.transform_point(p)),
            LineCommand::LineBy(v) => LineCommand::LineBy(transformation.transform_vector(v)),
            LineCommand::HorizontalLineTo(x) => LineCommand::HorizontalLineTo(
                transformation.transform_point(point2(x, T::zero())).x,
            ),
            LineCommand::HorizontalLineBy(x) => {
                LineCommand::HorizontalLineBy(transformation.transform_vector(vec2(x, T::zero())).x)
            }
            LineCommand::VerticalLineTo(y) => {
                LineCommand::VerticalLineTo(transformation.transform_point(point2(T::zero(), y)).y)
            }
            LineCommand::VerticalLineBy(y) => {
                LineCommand::VerticalLineBy(transformation.transform_vector(vec2(T::zero(), y)).y)
            }
            LineCommand::Close => LineCommand::Close,
            LineCommand::CubicBezierTo {
                control_1,
                control_2,
                end,
            } => LineCommand::CubicBezierTo {
                control_1: transformation.transform_point(control_1),
                control_2: transformation.transform_point(control_2),
                end: transformation.transform_point(end),
            },
            LineCommand::CubicBezierBy {
                control_1,
                control_2,
                end,
            } => LineCommand::CubicBezierBy {
                control_1: transformation.transform_vector(control_1),
                control_2: transformation.transform_vector(control_2),
                end: transformation.transform_vector(end),
            },
            LineCommand::SmoothCubicBezierTo { control, end } => LineCommand::SmoothCubicBezierTo {
                control: transformation.transform_point(control),
                end: transformation.transform_point(end),
            },
            LineCommand::SmoothCubicBezierBy { control, end } => LineCommand::SmoothCubicBezierBy {
                control: transformation.transform_vector(control),
                end: transformation.transform_vector(end),
            },
            LineCommand::QuadraticBezierTo { control, end } => LineCommand::QuadraticBezierTo {
                control: transformation.transform_point(control),
                end: transformation.transform_point(end),
            },
            LineCommand::QuadraticBezierBy { control, end } => LineCommand::QuadraticBezierBy {
                control: transformation.transform_vector(control),
                end: transformation.transform_vector(end),
            },
            LineCommand::SmoothQuadtraticCurveTo(p) => {
                LineCommand::SmoothQuadtraticCurveTo(transformation.transform_point(p))
            }
            LineCommand::SmoothQuadtraticCurveBy(v) => {
                LineCommand::SmoothQuadtraticCurveBy(transformation.transform_vector(v))
            }
            LineCommand::ArcTo {
                x_radius,
                y_radius,
                x_axis_rotation,
                large_arc_flag,
                sweep_flag,
                end,
            } => {
                let radius = transformation.transform_point(point2(x_radius, y_radius));
                LineCommand::ArcTo {
                    x_radius: radius.x,
                    y_radius: radius.y,
                    x_axis_rotation,
                    large_arc_flag,
                    sweep_flag,
                    end: transformation.transform_point(end),
                }
            }
            LineCommand::ArcBy {
                x_radius,
                y_radius,
                x_axis_rotation,
                large_arc_flag,
                sweep_flag,
                end,
            } => {
                let radius = transformation.transform_vector(vec2(x_radius, y_radius));
                LineCommand::ArcBy {
                    x_radius: radius.x,
                    y_radius: radius.y,
                    x_axis_rotation,
                    large_arc_flag,
                    sweep_flag,
                    end: transformation.transform_vector(end),
                }
            }
        }
    }

    /// Transform this line command in place with the given linear transformation.
    pub fn transform_in_place(&mut self, transformation: &euclid::Transform2D<T, U, U>) {
        *self = self.transform(transformation);
    }
}

impl<T, U> LineCommand<T, U>
where
    T: Copy + NumCast,
{
    /// Cast from number representation `T` to number representation `V`.
    #[inline]
    pub fn cast<V>(&self) -> LineCommand<V, U>
    where
        V: NumCast + Copy,
    {
        match *self {
            LineCommand::MoveTo(p) => LineCommand::MoveTo(p.cast()),
            LineCommand::MoveBy(v) => LineCommand::MoveBy(v.cast()),
            LineCommand::LineTo(p) => LineCommand::LineTo(p.cast()),
            LineCommand::LineBy(v) => LineCommand::LineBy(v.cast()),
            LineCommand::HorizontalLineTo(x) => LineCommand::HorizontalLineTo(V::from(x).unwrap()),
            LineCommand::HorizontalLineBy(x) => LineCommand::HorizontalLineBy(V::from(x).unwrap()),
            LineCommand::VerticalLineTo(y) => LineCommand::VerticalLineTo(V::from(y).unwrap()),
            LineCommand::VerticalLineBy(y) => LineCommand::VerticalLineBy(V::from(y).unwrap()),
            LineCommand::Close => LineCommand::Close,
            LineCommand::CubicBezierTo {
                control_1,
                control_2,
                end,
            } => LineCommand::CubicBezierTo {
                control_1: control_1.cast(),
                control_2: control_2.cast(),
                end: end.cast(),
            },
            LineCommand::CubicBezierBy {
                control_1,
                control_2,
                end,
            } => LineCommand::CubicBezierBy {
                control_1: control_1.cast(),
                control_2: control_2.cast(),
                end: end.cast(),
            },
            LineCommand::SmoothCubicBezierTo { control, end } => LineCommand::SmoothCubicBezierTo {
                control: control.cast(),
                end: end.cast(),
            },
            LineCommand::SmoothCubicBezierBy { control, end } => LineCommand::SmoothCubicBezierBy {
                control: control.cast(),
                end: end.cast(),
            },
            LineCommand::QuadraticBezierTo { control, end } => LineCommand::QuadraticBezierTo {
                control: control.cast(),
                end: end.cast(),
            },
            LineCommand::QuadraticBezierBy { control, end } => LineCommand::QuadraticBezierBy {
                control: control.cast(),
                end: end.cast(),
            },
            LineCommand::SmoothQuadtraticCurveTo(p) => {
                LineCommand::SmoothQuadtraticCurveTo(p.cast())
            }
            LineCommand::SmoothQuadtraticCurveBy(v) => {
                LineCommand::SmoothQuadtraticCurveBy(v.cast())
            }
            LineCommand::ArcTo {
                x_radius,
                y_radius,
                x_axis_rotation,
                large_arc_flag,
                sweep_flag,
                end,
            } => {
                let radius: Point2D<V, U> = point2::<T, U>(x_radius, y_radius).cast();
                LineCommand::ArcTo {
                    x_radius: radius.x,
                    y_radius: radius.y,
                    x_axis_rotation: euclid::Angle::radians(
                        V::from(x_axis_rotation.get()).unwrap(),
                    ),
                    large_arc_flag,
                    sweep_flag,
                    end: end.cast(),
                }
            }
            LineCommand::ArcBy {
                x_radius,
                y_radius,
                x_axis_rotation,
                large_arc_flag,
                sweep_flag,
                end,
            } => {
                let radius: Vector2D<V, U> = vec2::<T, U>(x_radius, y_radius).cast();
                LineCommand::ArcBy {
                    x_radius: radius.x,
                    y_radius: radius.y,
                    x_axis_rotation: V::from(x_axis_rotation).unwrap(),
                    large_arc_flag,
                    sweep_flag,
                    end: end.cast(),
                }
            }
        }
    }
}

impl<T, U> Path<T, U> {
    /// Construct a new, empty path.
    pub fn new() -> Path<T, U> {
        Default::default()
    }

    /// Construct a new path with the given line commands.
    pub fn with_commands<I>(commands: I) -> Path<T, U>
    where
        I: IntoIterator<Item = LineCommand<T, U>>,
    {
        Path {
            color: "black".into(),
            commands: commands.into_iter().collect(),
        }
    }
}

impl<T, U> Path<T, U>
where
    T: Copy + NumCast,
{
    /// Cast from number representation `T` to number representation `V`.
    #[inline]
    pub fn cast<V>(&self) -> Path<V, U>
    where
        V: NumCast + Copy,
    {
        Path {
            color: self.color.clone(),
            commands: self.commands.iter().map(|c| c.cast::<V>()).collect(),
        }
    }
}

impl<T, U> Path<T, U>
where
    T: Copy + Num + PartialOrd,
{
    /// Transform this path with the given linear transformation and return
    /// the new, transformed path.
    pub fn transform<V>(&self, transformation: &euclid::Transform2D<T, U, V>) -> Path<T, V> {
        Path {
            color: self.color.clone(),
            commands: self
                .commands
                .iter()
                .map(|c| c.transform(transformation))
                .collect(),
        }
    }

    /// Transform this path in place with the given linear transformation.
    pub fn transform_in_place(&mut self, transformation: &euclid::Transform2D<T, U, U>) {
        for c in &mut self.commands {
            c.transform_in_place(transformation);
        }
    }
}

impl<'a, T> From<&'a Path<T, CanvasSpace>> for svg::node::element::Path
where
    T: Copy + Debug + Into<svg::node::element::path::Parameters>,
{
    fn from(path: &'a Path<T, CanvasSpace>) -> svg::node::element::Path {
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
                LineCommand::QuadraticBezierTo { control, end } => {
                    data.quadratic_curve_to((control.x, control.y, end.x, end.y))
                }
                LineCommand::QuadraticBezierBy { control, end } => {
                    data.quadratic_curve_by((control.x, control.y, end.x, end.y))
                }
                LineCommand::SmoothCubicBezierTo { control, end } => {
                    data.smooth_cubic_curve_to((control.x, control.y, end.x, end.y))
                }
                LineCommand::SmoothCubicBezierBy { control, end } => {
                    data.smooth_cubic_curve_by((control.x, control.y, end.x, end.y))
                }
                LineCommand::SmoothQuadtraticCurveTo(p) => {
                    data.smooth_quadratic_curve_to((p.x, p.y))
                }
                LineCommand::SmoothQuadtraticCurveBy(v) => {
                    data.smooth_quadratic_curve_by((v.x, v.y))
                }
                cmd => unimplemented!("Have not implemented support for command yet: {:?}", cmd),
            };
        }
        svg::node::element::Path::new()
            .set("stroke", path.color.as_str())
            .set("fill", "none")
            .set("d", data)
    }
}

/// Anything that can render as one or more `Path`s.
///
/// This is implemented for `fart::geom::Polygon` and
/// `fart::geom::ConvexPolygon`, and draws the outline of the polygon. It is
/// also implemented for `fart::geom::Line`, and draws just the line.
pub trait ToPaths<T, U> {
    /// An iterator over this thing's paths.
    type Paths: Iterator<Item = Path<T, U>>;

    /// Render this thing as `Path`s.
    fn to_paths(&self) -> Self::Paths;
}

impl<'a, P, T, U> ToPaths<T, U> for &'a P
where
    P: ToPaths<T, U>,
{
    type Paths = <P as ToPaths<T, U>>::Paths;

    fn to_paths(&self) -> Self::Paths {
        (*self).to_paths()
    }
}

impl<T, U> ToPaths<T, U> for fart_2d_geom::Polygon<T, U>
where
    T: Copy + NumAssign + PartialOrd + Signed + Debug,
{
    type Paths = iter::Once<Path<T, U>>;

    fn to_paths(&self) -> Self::Paths {
        let mut commands = Vec::with_capacity(self.vertices().len() + 1);

        let mut first = true;
        for v in self.vertices() {
            commands.push(if first {
                first = false;
                LineCommand::MoveTo(*v)
            } else {
                LineCommand::LineTo(*v)
            });
        }
        commands.push(LineCommand::Close);

        iter::once(Path::with_commands(commands))
    }
}

impl<T, U> ToPaths<T, U> for fart_2d_geom::ConvexPolygon<T, U>
where
    T: Copy + NumAssign + PartialOrd + Signed + Debug,
{
    type Paths = <fart_2d_geom::Polygon<T, U> as ToPaths<T, U>>::Paths;

    fn to_paths(&self) -> Self::Paths {
        let p: &fart_2d_geom::Polygon<T, U> = self.as_ref();
        p.to_paths()
    }
}

impl<T, U> ToPaths<T, U> for fart_2d_geom::Line<T, U>
where
    T: Clone,
{
    type Paths = iter::Once<Path<T, U>>;

    fn to_paths(&self) -> Self::Paths {
        iter::once(Path::with_commands(vec![
            LineCommand::MoveTo(self.a.clone()),
            LineCommand::LineTo(self.b.clone()),
        ]))
    }
}

impl<T, U> ToPaths<T, U> for fart_aabb::Aabb<T, U>
where
    T: Copy + Num + PartialOrd,
{
    type Paths = iter::Once<Path<T, U>>;

    fn to_paths(&self) -> Self::Paths {
        iter::once(Path::with_commands(vec![
            LineCommand::MoveTo(self.min()),
            LineCommand::LineTo(point2(self.max().x, self.min().y)),
            LineCommand::LineTo(self.max()),
            LineCommand::LineTo(point2(self.min().x, self.max().y)),
            LineCommand::Close,
        ]))
    }
}

/// A `ToPaths` wrapper type that forces all of the paths produced by the inner
/// type to be of a certain color. Created using `ToPathsExt::color`.
#[derive(Debug, Clone)]
pub struct Color<P> {
    inner: P,
    color: Cow<'static, str>,
}

/// An iterator over paths produced by `Color<P>`. Created via `<Color<P> as
/// ToPaths<_, _>>::to_paths()`.
#[derive(Clone, Debug)]
pub struct ColorPaths<P, T, U> {
    inner: P,
    color: Cow<'static, str>,
    _phantom: PhantomData<fn() -> Path<T, U>>,
}

impl<P, T, U> Iterator for ColorPaths<P, T, U>
where
    P: Iterator<Item = Path<T, U>>,
{
    type Item = Path<T, U>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut path = self.inner.next()?;
        path.color = self.color.clone().into();
        Some(path)
    }
}

impl<P, T, U> ToPaths<T, U> for Color<P>
where
    P: ToPaths<T, U>,
{
    type Paths = ColorPaths<P::Paths, T, U>;

    fn to_paths(&self) -> Self::Paths {
        let inner = self.inner.to_paths();
        let color = self.color.clone();
        ColorPaths {
            inner,
            color,
            _phantom: PhantomData,
        }
    }
}

/// An extension trait for shapes to add various helper utilities.
pub trait ToPathsExt<T, U>: ToPaths<T, U> {
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

impl<S, T, U> ToPathsExt<T, U> for S where S: ToPaths<T, U> {}
