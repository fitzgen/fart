//! Two-dimensional geometry algorithms built on top of `euclid`.

#![deny(missing_docs, missing_debug_implementations)]

use euclid::TypedPoint2D;
use num_traits::{Num, NumAssign, Signed};
use partial_min_max::{max, min};

fn area2<T, U>(a: TypedPoint2D<T, U>, b: TypedPoint2D<T, U>, c: TypedPoint2D<T, U>) -> T
where
    T: Copy + Num,
{
    (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)
}

/// A polygon.
///
/// The polygon's points are in counter-clockwise order.
///
/// No guarantees whether this polygon is convex or not.
///
/// * `T` is the numeric type. `i32` or `f64` etc.
/// * `U` is the unit. `ScreenSpace` or `WorldSpace` etc.
#[derive(Clone, Debug)]
pub struct Polygon<T, U> {
    points: Vec<TypedPoint2D<T, U>>,
}

impl<T, U> Polygon<T, U>
where
    T: Copy + NumAssign,
{
    /// Construct a new polygon.
    pub fn new(points: Vec<TypedPoint2D<T, U>>) -> Polygon<T, U> {
        assert!(points.len() >= 3);
        Polygon { points }
    }

    /// Get the area of this polygon.
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use euclid_2d_geom::Polygon;
    ///
    /// let square: Polygon<i32, UnknownUnit> = Polygon::new(vec![
    ///     point2(0, 0),
    ///     point2(10, 0),
    ///     point2(10, 10),
    ///     point2(0, 10),
    /// ]);
    ///
    /// assert_eq!(square.area(), 100);
    ///
    /// let triangle: Polygon<i32, UnknownUnit> = Polygon::new(vec![
    ///     point2(-6, -6),
    ///     point2(6, 0),
    ///     point2(0, 0),
    /// ]);
    ///
    /// assert_eq!(triangle.area(), 18);
    /// ```
    #[inline]
    pub fn area(&self) -> T
    where
        T: Signed,
    {
        let two = T::one() + T::one();
        (self.signed_double_area() / two).abs()
    }

    fn signed_double_area(&self) -> T
    where
        T: Signed,
    {
        let mut sum = T::zero();
        for i in 1..self.points.len() - 1 {
            sum += area2(self.points[0], self.points[i], self.points[i + 1]);
        }
        sum
    }
}

/// A line between two points.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Line<T, U> {
    /// The first point.
    pub a: TypedPoint2D<T, U>,
    /// The second point.
    pub b: TypedPoint2D<T, U>,
}

/// Convenience function for creating lines.
#[inline]
pub fn line<T, U>(a: TypedPoint2D<T, U>, b: TypedPoint2D<T, U>) -> Line<T, U> {
    Line { a, b }
}

impl<T, U> Line<T, U>
where
    T: Copy + Num + PartialOrd,
{
    /// Create a new line between the given points.
    #[inline]
    pub fn new(a: TypedPoint2D<T, U>, b: TypedPoint2D<T, U>) -> Line<T, U> {
        line(a, b)
    }

    /// Is the given point on the left of this line?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use euclid_2d_geom::{line, Line};
    ///
    /// let l: Line<i32, UnknownUnit> = line(point2(0, 0), point2(1, 1));
    ///
    /// assert!(l.is_left(point2(0, 1)));
    /// assert!(!l.is_left(point2(1, 0)));
    ///
    /// // Collinear points are not considered on the left of the line. See
    /// // also `is_left_or_on`.
    /// assert!(!l.is_left(point2(2, 2)));
    /// ```
    #[inline]
    pub fn is_left(&self, point: TypedPoint2D<T, U>) -> bool {
        area2(self.a, self.b, point) > T::zero()
    }

    /// Is the given point on the left of this line or collinear with it?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use euclid_2d_geom::{line, Line};
    ///
    /// let l: Line<i32, UnknownUnit> = line(point2(0, 0), point2(1, 1));
    ///
    /// assert!(l.is_left_or_collinear(point2(0, 1)));
    /// assert!(l.is_left_or_collinear(point2(2, 2)));
    ///
    /// assert!(!l.is_left_or_collinear(point2(1, 0)));
    /// ```
    #[inline]
    pub fn is_left_or_collinear(&self, point: TypedPoint2D<T, U>) -> bool {
        area2(self.a, self.b, point) >= T::zero()
    }

    /// Is the given point collinear with this line?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use euclid_2d_geom::{line, Line};
    ///
    /// let l: Line<i32, UnknownUnit> = line(point2(0, 0), point2(1, 1));
    ///
    /// assert!(l.is_collinear(point2(2, 2)));
    ///
    /// assert!(!l.is_collinear(point2(0, 1)));
    /// assert!(!l.is_collinear(point2(1, 0)));
    /// ```
    #[inline]
    pub fn is_collinear(&self, point: TypedPoint2D<T, U>) -> bool {
        area2(self.a, self.b, point) == T::zero()
    }

    /// Is the given point on this line segment? That is, not just collinear,
    /// but also between `self.a` and `self.b`?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use euclid_2d_geom::{line, Line};
    ///
    /// let l: Line<i32, UnknownUnit> = line(point2(0, 0), point2(2, 2));
    ///
    /// assert!(l.is_on(point2(1, 1)));
    ///
    /// assert!(!l.is_on(point2(0, 1)));
    /// assert!(!l.is_on(point2(1, 0)));
    ///
    /// // Inclusive of the line segment's boundaries.
    /// assert!(l.is_on(l.a));
    /// assert!(l.is_on(l.b));
    ///
    /// // Does not include collinear-but-not-between points.
    /// assert!(!l.is_on(point2(3, 3)));
    /// ```
    pub fn is_on(&self, point: TypedPoint2D<T, U>) -> bool {
        if !self.is_collinear(point) {
            return false;
        }

        // If this line segment is vertical, check that point.y is between a.y
        // and b.y. Otherwise check that point.x is between a.x and b.x.
        if self.a.x == self.b.x {
            let min = min(self.a.y, self.b.y);
            let max = max(self.a.y, self.b.y);
            min <= point.y && point.y <= max
        } else {
            let min = min(self.a.x, self.b.x);
            let max = max(self.a.x, self.b.x);
            min <= point.x && point.x <= max
        }
    }

    /// Does this line segment (properly) intersect with the other line segment?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use euclid_2d_geom::{line, Line};
    ///
    /// assert!(
    ///     line::<i32, UnknownUnit>(point2(0, 0), point2(1, 1))
    ///         .intersects(&line(point2(0, 1), point2(1, 0)))
    /// );
    ///
    /// assert!(
    ///     !line::<i32, UnknownUnit>(point2(0, 0), point2(1, 1))
    ///         .intersects(&line(point2(1, 0), point2(2, -1)))
    /// );
    ///
    /// // If any end points from one line segment land on the other line
    /// // segment, `false` is returned because that is not proper intersection.
    /// assert!(
    ///     !line::<i32, UnknownUnit>(point2(0, 0), point2(2, 2))
    ///         .intersects(&line(point2(1, 1), point2(2, 0)))
    /// );
    /// ```
    #[inline]
    pub fn intersects(&self, other: &Line<T, U>) -> bool {
        // If any points from a line segment are collinear with the other line
        // segment, then they cannot properly intersect.
        if self.is_collinear(other.a)
            || self.is_collinear(other.b)
            || other.is_collinear(self.a)
            || other.is_collinear(self.b)
        {
            return false;
        }

        (self.is_left(other.a) ^ self.is_left(other.b))
            && (other.is_left(self.a) ^ other.is_left(self.b))
    }
}
