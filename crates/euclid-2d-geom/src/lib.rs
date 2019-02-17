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
/// The polygon's vertices are in counter-clockwise order.
///
/// No guarantees whether this polygon is convex or not.
///
/// * `T` is the numeric type. `i32` or `f64` etc.
/// * `U` is the unit. `ScreenSpace` or `WorldSpace` etc.
#[derive(Clone, Debug)]
pub struct Polygon<T, U> {
    vertices: Vec<TypedPoint2D<T, U>>,
}

impl<T, U> Polygon<T, U>
where
    T: Copy + NumAssign + PartialOrd,
{
    /// Construct a new polygon.
    pub fn new(vertices: Vec<TypedPoint2D<T, U>>) -> Polygon<T, U> {
        assert!(vertices.len() >= 3);
        Polygon { vertices }
    }

    /// Get the `i`th point in this polygon.
    pub fn get(&self, i: usize) -> Option<TypedPoint2D<T, U>> {
        self.vertices.get(i).cloned()
    }

    /// Get the number of vertices in this polygon.
    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    /// Get the index of the next vertex after `i` in this polygon.
    #[inline]
    pub fn next(&self, i: usize) -> usize {
        assert!(i < self.vertices.len());
        let next = i + 1;
        if next == self.vertices.len() {
            0
        } else {
            next
        }
    }

    /// Get the index of the previous vertex after `i` in this polygon.
    #[inline]
    pub fn prev(&self, i: usize) -> usize {
        assert!(i < self.vertices.len());
        if i == 0 {
            self.vertices.len() - 1
        } else {
            i - 1
        }
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
        for i in 1..self.vertices.len() - 1 {
            sum += area2(self.vertices[0], self.vertices[i], self.vertices[i + 1]);
        }
        sum
    }

    /// Do the `a`th and `b`th vertices within this polygon form a diagonal?
    ///
    /// If `a` and `b` are diagonal, then there is a direct line of sight
    /// between them, and they are internal to this polygon.
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use euclid_2d_geom::Polygon;
    ///
    /// let p: Polygon<i32, UnknownUnit> = Polygon::new(vec![
    ///     point2(0,  0),
    ///     point2(10, 0),
    ///     point2(5,  5),
    ///     point2(10, 10),
    ///     point2(0,  10),
    /// ]);
    ///
    /// assert!(p.is_diagonal(0, 2));
    /// assert!(!p.is_diagonal(1, 3));
    /// ```
    pub fn is_diagonal(&self, a: usize, b: usize) -> bool {
        assert!(a < self.vertices.len());
        assert!(b < self.vertices.len());

        self.in_cone(a, b) && self.in_cone(b, a) && self.internal_or_external_diagonal(a, b)
    }

    // Is `b` within the cone from `prev(a)` to `a` to `next(a)`?
    #[inline]
    fn in_cone(&self, a: usize, b: usize) -> bool {
        assert!(a < self.vertices.len());
        assert!(b < self.vertices.len());

        let a_prev = self.vertices[self.prev(a)];
        let a_next = self.vertices[self.next(a)];
        let a = self.vertices[a];
        let b = self.vertices[b];

        // If `a_prev` is left of the line from `a` to `a_next`, then the cone
        // is convex.
        if line(a, a_next).is_left(a_prev) {
            // When the cone is convex, we just need to check that `a_prev` is
            // left of the line `a` to `b` and the `a_next` is to the right.
            let l = line(a, b);
            l.is_left(a_prev) && l.is_right(a_next)
        } else {
            // When the cone is reflex, we check that it is *not* in the inverse
            // of the cone. The inverse cone is convex, and therefore can be
            // checked as in the above case, except we allow collinearity since
            // we then negate the whole thing.
            let l = line(a, b);
            !(l.is_left_or_collinear(a_next) && l.is_right_or_collinear(a_prev))
        }
    }

    fn internal_or_external_diagonal(&self, a: usize, b: usize) -> bool {
        assert!(a < self.vertices.len());
        assert!(b < self.vertices.len());

        let l = line(self.vertices[a], self.vertices[b]);

        for (i, j) in (0..self.vertices.len()).zip((1..self.vertices.len()).chain(Some(0))) {
            if i == a || i == b || j == a || j == b {
                continue;
            }

            let m = line(self.vertices[i], self.vertices[j]);
            if l.improperly_intersects(&m) {
                return false;
            }
        }

        true
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
    /// // also `is_left_or_collinear`.
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

    /// Is the given point on the right of this line?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use euclid_2d_geom::{line, Line};
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
    pub fn is_right(&self, point: TypedPoint2D<T, U>) -> bool {
        area2(self.a, self.b, point) < T::zero()
    }

    /// Is the given point on the right of this line or collinear with it?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use euclid_2d_geom::{line, Line};
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
        area2(self.a, self.b, point) <= T::zero()
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

    /// Does this line segment improperly intersect with the other line segment?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use euclid_2d_geom::{line, Line};
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
