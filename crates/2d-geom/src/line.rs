use crate::area2;
use euclid::{point2, Point2D};
use fart_aabb::{Aabb, ToAabb};
use fart_utils::NoMorePartial;
use num_traits::Num;
use partial_min_max::{max, min};
use std::cmp::Ordering;

/// A line between two points.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Line<T, U> {
    /// The first point.
    pub a: Point2D<T, U>,
    /// The second point.
    pub b: Point2D<T, U>,
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
pub fn line<T, U>(a: Point2D<T, U>, b: Point2D<T, U>) -> Line<T, U> {
    Line { a, b }
}

impl<T, U> Line<T, U>
where
    T: Copy + Num + PartialOrd,
{
    /// Create a new line between the given points.
    #[inline]
    pub fn new(a: Point2D<T, U>, b: Point2D<T, U>) -> Line<T, U> {
        line(a, b)
    }

    /// Get the direction of the point relative to this line.
    #[inline]
    pub fn relative_direction_of(&self, point: Point2D<T, U>) -> RelativeDirection {
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
    pub fn is_left(&self, point: Point2D<T, U>) -> bool {
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
    pub fn is_left_or_collinear(&self, point: Point2D<T, U>) -> bool {
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
    pub fn is_collinear(&self, point: Point2D<T, U>) -> bool {
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
    pub fn is_right(&self, point: Point2D<T, U>) -> bool {
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
    pub fn is_right_or_collinear(&self, point: Point2D<T, U>) -> bool {
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
    pub fn is_on(&self, point: Point2D<T, U>) -> bool {
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

impl<U> Line<f64, U> {
    /// Get the intersection between two line segments.
    ///
    /// The kind of intersection is broken down by whether it is proper,
    /// improper, or collinear. If you don't care what kind of intersection it
    /// is, use `LineIntersection::point` to just get the point of intersection,
    /// if any.
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::{Line, LineIntersection};
    ///
    /// let line = |a, b| -> Line<f64, UnknownUnit> {
    ///     Line::new(a, b)
    /// };
    ///
    /// // No intersection.
    /// assert_eq!(
    ///     line(
    ///         point2(0.0, 0.0),
    ///         point2(5.0, 5.0),
    ///     ).intersection(&line(
    ///         point2(0.0, 3.0),
    ///         point2(1.0, 3.0),
    ///     )),
    ///     LineIntersection::None,
    /// );
    ///
    /// // Proper intersection.
    /// assert_eq!(
    ///     line(
    ///         point2(0.0, 0.0),
    ///         point2(5.0, 5.0),
    ///     ).intersection(&line(
    ///         point2(0.0, 2.0),
    ///         point2(2.0, 0.0),
    ///     )),
    ///     LineIntersection::Proper(point2(1.0, 1.0)),
    /// );
    ///
    /// // Improper intersection.
    /// assert_eq!(
    ///     line(
    ///         point2(0.0, 0.0),
    ///         point2(5.0, 5.0),
    ///     ).intersection(&line(
    ///         point2(5.0, 5.0),
    ///         point2(2.0, 0.0),
    ///     )),
    ///     LineIntersection::Improper(point2(5.0, 5.0)),
    /// );
    ///
    /// // Collinear intersection.
    /// assert_eq!(
    ///     line(
    ///         point2(0.0, 0.0),
    ///         point2(5.0, 5.0),
    ///     ).intersection(&line(
    ///         point2(3.0, 3.0),
    ///         point2(7.0, 7.0),
    ///     )),
    ///     LineIntersection::Collinear(point2(3.0, 3.0)),
    /// );
    ///
    /// // Don't care what kind, just give me the point!
    /// assert_eq!(
    ///     line(
    ///         point2(0.0, 0.0),
    ///         point2(5.0, 5.0),
    ///     ).intersection(&line(
    ///         point2(0.0, 3.0),
    ///         point2(1.0, 3.0),
    ///     )).point(),
    ///     None,
    /// );
    /// assert_eq!(
    ///     line(
    ///         point2(0.0, 0.0),
    ///         point2(5.0, 5.0),
    ///     ).intersection(&line(
    ///         point2(0.0, 2.0),
    ///         point2(2.0, 0.0),
    ///     )).point(),
    ///     Some(point2(1.0, 1.0)),
    /// );
    /// ```
    pub fn intersection(&self, other: &Line<f64, U>) -> LineIntersection<U> {
        let denominator = self.a.x * (other.b.y - other.a.y)
            + self.b.x * (other.a.y - other.b.y)
            + other.b.x * (self.b.y - self.a.y)
            + other.a.x * (self.a.y - self.b.y);

        if denominator == 0.0 {
            return self.parallel_intersection(other);
        }

        let numerator = self.a.x * (other.b.y - other.a.y)
            + other.a.x * (self.a.y - other.b.y)
            + other.b.x * (other.a.y - self.a.y);

        let s = numerator / denominator;

        let numerator = -(self.a.x * (other.a.y - self.b.y)
            + self.b.x * (self.a.y - other.a.y)
            + other.a.x * (self.b.y - self.a.y));

        let t = numerator / denominator;

        let p = self.a.lerp(self.b, s);

        if numerator == 0.0 || numerator == denominator {
            LineIntersection::Improper(p)
        } else if 0.0 < s && s < 1.0 && 0.0 < t && t < 1.0 {
            LineIntersection::Proper(p)
        } else {
            LineIntersection::None
        }
    }

    fn parallel_intersection(&self, other: &Line<f64, U>) -> LineIntersection<U> {
        let between = |l: &Self, p: euclid::Point2D<f64, U>| {
            if l.a.x != l.b.x {
                (l.a.x <= p.x && p.x <= l.b.x) || (l.b.x <= p.x && p.x <= l.a.x)
            } else {
                (l.a.y <= p.y && p.y <= l.b.y) || (l.b.y <= p.y && p.y <= l.a.y)
            }
        };

        if !self.is_collinear(other.a) {
            LineIntersection::None
        } else if between(self, other.a) {
            LineIntersection::Collinear(other.a)
        } else if between(self, other.b) {
            LineIntersection::Collinear(other.a)
        } else if between(other, self.a) {
            LineIntersection::Collinear(self.a)
        } else if between(other, self.b) {
            LineIntersection::Collinear(self.b)
        } else {
            LineIntersection::None
        }
    }
}

/// The result of `Line::intersection` providing the intersection point between
/// two line segments, if any.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LineIntersection<U> {
    /// The line segments do not intersect.
    None,

    /// The line segments properly intersect at the given point, and are not
    /// collinear.
    Proper(euclid::Point2D<f64, U>),

    /// The line segments improperly intersect and are not collinear, with the
    /// endpoint of one line segment landing on the other.
    Improper(euclid::Point2D<f64, U>),

    /// The lines are collinear and intersect at the given point (and perhaps
    /// infinitely many other points as well).
    Collinear(euclid::Point2D<f64, U>),
}

impl<U> LineIntersection<U> {
    /// Is this a `LineIntersection::None`?
    #[inline]
    pub fn is_none(&self) -> bool {
        match self {
            LineIntersection::None => true,
            _ => false,
        }
    }

    /// Is this a `LineIntersection::Proper`?
    #[inline]
    pub fn is_proper(&self) -> bool {
        match self {
            LineIntersection::Proper(_) => true,
            _ => false,
        }
    }

    /// Is this a `LineIntersection::Improper`?
    #[inline]
    pub fn is_improper(&self) -> bool {
        match self {
            LineIntersection::Improper(_) => true,
            _ => false,
        }
    }

    /// Is this a `LineIntersection::Collinear`?
    #[inline]
    pub fn is_collinear(&self) -> bool {
        match self {
            LineIntersection::Collinear(_) => true,
            _ => false,
        }
    }

    /// Get the intersection point, if any, regardless if this is a proper,
    /// improper, or collinear intersection.
    #[inline]
    pub fn point(&self) -> Option<euclid::Point2D<f64, U>> {
        match *self {
            LineIntersection::None => None,
            LineIntersection::Proper(p)
            | LineIntersection::Improper(p)
            | LineIntersection::Collinear(p) => Some(p),
        }
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
