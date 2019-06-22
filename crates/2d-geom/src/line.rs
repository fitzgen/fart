use crate::area2;
use euclid::{point2, TypedPoint2D};
use fart_aabb::{Aabb, ToAabb};
use fart_utils::NoMorePartial;
use num_traits::Num;
use partial_min_max::{max, min};
use std::cmp::Ordering;

/// A line between two points.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Line<T, U> {
    /// The first point.
    pub a: TypedPoint2D<T, U>,
    /// The second point.
    pub b: TypedPoint2D<T, U>,
}

/// The direction a point lies relative to a line. Returned by
/// `Line::relative_direction_of`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelativeDirection {
    /// The point lies left relative to the line.
    Left = 1,

    /// The point is collinear with the line.
    Collinear = 0,

    /// The point lies right relative to the line.
    Right = -1,
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

    /// Get the direction of the point relative to this line.
    #[inline]
    pub fn relative_direction_of(&self, point: TypedPoint2D<T, U>) -> RelativeDirection {
        let zero = NoMorePartial(T::zero());
        let det = NoMorePartial(area2(self.a, self.b, point));
        match det.cmp(&zero) {
            Ordering::Greater => RelativeDirection::Left,
            Ordering::Equal => RelativeDirection::Collinear,
            Ordering::Less => RelativeDirection::Right,
        }
    }

    /// Is the given point on the left of this line?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::{line, Line};
    ///
    /// let l: Line<i32, UnknownUnit> = line(point2(0, 0), point2(1, 1));
    ///
    /// assert!(l.is_left(point2(0, 1)));
    /// assert!(!l.is_left(point2(1, 0)));
    ///
    /// // Collinear points are not considered on the left of the line. See
    /// // also `is_left_or_collinear`.
    /// assert!(!l.is_left(point2(2, 2)));
    /// ```
    #[inline]
    pub fn is_left(&self, point: TypedPoint2D<T, U>) -> bool {
        self.relative_direction_of(point) == RelativeDirection::Left
    }

    /// Is the given point on the left of this line or collinear with it?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::{line, Line};
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
        match self.relative_direction_of(point) {
            RelativeDirection::Left | RelativeDirection::Collinear => true,
            RelativeDirection::Right => false,
        }
    }

    /// Is the given point collinear with this line?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::{line, Line};
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
        self.relative_direction_of(point) == RelativeDirection::Collinear
    }

    /// Is the given point on the right of this line?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::{line, Line};
    ///
    /// let l: Line<i32, UnknownUnit> = line(point2(0, 0), point2(1, 1));
    ///
    /// assert!(l.is_right(point2(1, 0)));
    /// assert!(!l.is_right(point2(0, 1)));
    ///
    /// // Collinear points are not considered on the right of the line. See
    /// // also `is_right_or_collinear`.
    /// assert!(!l.is_right(point2(2, 2)));
    /// ```
    #[inline]
    pub fn is_right(&self, point: TypedPoint2D<T, U>) -> bool {
        self.relative_direction_of(point) == RelativeDirection::Right
    }

    /// Is the given point on the right of this line or collinear with it?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::{line, Line};
    ///
    /// let l: Line<i32, UnknownUnit> = line(point2(0, 0), point2(1, 1));
    ///
    /// assert!(l.is_right_or_collinear(point2(1, 0)));
    /// assert!(l.is_right_or_collinear(point2(2, 2)));
    ///
    /// assert!(!l.is_right_or_collinear(point2(0, 1)));
    /// ```
    #[inline]
    pub fn is_right_or_collinear(&self, point: TypedPoint2D<T, U>) -> bool {
        match self.relative_direction_of(point) {
            RelativeDirection::Right | RelativeDirection::Collinear => true,
            RelativeDirection::Left => false,
        }
    }

    /// Is the given point on this line segment? That is, not just collinear,
    /// but also between `self.a` and `self.b`?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::{line, Line};
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
    /// use fart_2d_geom::{line, Line};
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

    /// Does this line segment improperly intersect with the other line segment?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::{line, Line};
    ///
    /// assert!(
    ///     line::<i32, UnknownUnit>(point2(0, 0), point2(1, 1))
    ///         .improperly_intersects(&line(point2(0, 1), point2(1, 0)))
    /// );
    ///
    /// assert!(
    ///     !line::<i32, UnknownUnit>(point2(0, 0), point2(1, 1))
    ///         .improperly_intersects(&line(point2(1, 0), point2(2, -1)))
    /// );
    ///
    /// // If any end points from one line segment land on the other line
    /// // segment, `true` is still returned.
    /// assert!(
    ///     line::<i32, UnknownUnit>(point2(0, 0), point2(2, 2))
    ///         .improperly_intersects(&line(point2(1, 1), point2(2, 0)))
    /// );
    /// ```
    pub fn improperly_intersects(&self, other: &Line<T, U>) -> bool {
        self.intersects(other)
            || self.is_on(other.a)
            || self.is_on(other.b)
            || other.is_on(self.a)
            || other.is_on(self.b)
    }
}

impl<T, U> ToAabb<T, U> for Line<T, U>
where
    T: Copy + Num + PartialOrd,
{
    fn to_aabb(&self) -> Aabb<T, U> {
        let min = point2(min(self.a.x, self.b.x), min(self.a.y, self.b.y));
        let max = point2(max(self.a.x, self.b.x), max(self.a.y, self.b.y));
        Aabb::new(min, max)
    }
}
