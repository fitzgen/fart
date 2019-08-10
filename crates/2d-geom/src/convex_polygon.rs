use crate::{line, sort_around, Polygon};
use euclid::{point2, Point2D};
use fart_aabb::{Aabb, ToAabb};
use fart_utils::NoMorePartial;
use num_traits::{Bounded, Num, NumAssign, NumCast, Signed};
use std::fmt;
use std::ops::Deref;

/// A convex polygon.
///
/// This is a thin newtype wrapper over `Polygon`, and dereferences to the
/// underlying `Polygon`, but it's guaranteed that this polygon is convex.
#[derive(Clone)]
pub struct ConvexPolygon<T, U> {
    inner: Polygon<T, U>,
}

impl<T, U> fmt::Debug for ConvexPolygon<T, U>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ConvexPolygon")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T, U> AsRef<Polygon<T, U>> for ConvexPolygon<T, U> {
    fn as_ref(&self) -> &Polygon<T, U> {
        &self.inner
    }
}

impl<T, U> Deref for ConvexPolygon<T, U> {
    type Target = Polygon<T, U>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// NB: No `DerefMut` implementation because mutating the inner polygon might
// remove the convexity.

impl<T, U> From<ConvexPolygon<T, U>> for Polygon<T, U> {
    #[inline]
    fn from(c: ConvexPolygon<T, U>) -> Polygon<T, U> {
        c.inner
    }
}

impl<T, U> ConvexPolygon<T, U>
where
    T: Copy + NumAssign + PartialOrd + Signed + Bounded + fmt::Debug,
{
    /// Compute the convex hull of the given vertices.
    ///
    /// If the convex hull is a polygon with non-zero area, return it. Otherwise
    /// return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::ConvexPolygon;
    /// use std::collections::HashSet;
    ///
    /// let hull = ConvexPolygon::<i32, UnknownUnit>::hull(vec![
    ///     point2(0, 0),
    ///     point2(0, 1),
    ///     point2(0, 2),
    ///     point2(1, 0),
    ///     point2(1, 1),
    ///     point2(1, 2),
    ///     point2(2, 0),
    ///     point2(2, 1),
    ///     point2(2, 2),
    /// ]).expect("should have a convex hull for non-collinear vertex sets");
    ///
    /// let actual_hull_vertices = hull.vertices().iter().cloned().collect::<HashSet<_>>();
    ///
    /// let expected_hull_vertices = vec![
    ///     point2(0, 0),
    ///     point2(0, 2),
    ///     point2(2, 0),
    ///     point2(2, 2)
    /// ].into_iter().collect::<HashSet<_>>();
    ///
    /// assert_eq!(actual_hull_vertices, expected_hull_vertices);
    ///
    /// // Returns `None` for empty and collinear sets.
    /// assert!(ConvexPolygon::<i32, UnknownUnit>::hull(vec![]).is_none());
    /// assert!(ConvexPolygon::<i32, UnknownUnit>::hull(vec![point2(0, 0)]).is_none());
    /// assert!(ConvexPolygon::<i32, UnknownUnit>::hull(vec![point2(0, 0), point2(1, 1)]).is_none());
    /// assert!(ConvexPolygon::<i32, UnknownUnit>::hull(vec![point2(0, 0), point2(1, 1), point2(2, 2)]).is_none());
    /// ```
    pub fn hull(mut vertices: Vec<Point2D<T, U>>) -> Option<ConvexPolygon<T, U>> {
        let max = vertices
            .iter()
            .cloned()
            .fold(point2(T::min_value(), T::min_value()), |a, b| {
                if NoMorePartial((a.x, a.y)) > NoMorePartial((b.x, b.y)) {
                    a
                } else {
                    b
                }
            });

        sort_around(max, &mut vertices);
        vertices.dedup();

        if vertices.len() < 3 {
            return None;
        }

        debug_assert_eq!(max, vertices.last().cloned().unwrap());
        let mut stack = vec![max, vertices[0]];
        let mut i = 1;
        while i < vertices.len() - 1 {
            assert!(stack.len() >= 2);
            let v = vertices[i];
            let l = line(stack[stack.len() - 2], stack[stack.len() - 1]);
            if l.is_left(v) {
                // This vertex is (likely) part of the hull! Add it to our
                // stack.
                stack.push(v);
                i += 1;
            } else if stack.len() == 2 {
                // The first two vertices in the stack are always part of the
                // hull, and therefore should never be reconsidered, so start
                // considering the next `i`th vertex.
                i += 1;
            } else {
                // The top of our stack is not part of the hull, so pop it from
                // the stack to uncommit it.
                stack.pop();
            }
        }

        if stack.len() < 3 {
            return None;
        }

        Some(ConvexPolygon {
            inner: Polygon::new(stack),
        })
    }

    /// Does this convex polygon properly contain the given point?
    ///
    /// # Example
    ///
    /// ```
    /// use euclid::point2;
    /// use fart_2d_geom::ConvexPolygon;
    ///
    /// let p = ConvexPolygon::<i32, ()>::hull(vec![
    ///     point2(0, 0),
    ///     point2(10, 2),
    ///     point2(5, 10),
    /// ]).unwrap();
    ///
    /// assert!(p.contains_point(point2(5, 5)));
    /// assert!(!p.contains_point(point2(-3, -3)));
    ///
    /// // Points exactly on the edge are not considered contained.
    /// assert!(!p.contains_point(point2(0, 0)));
    /// ```
    pub fn contains_point(&self, point: Point2D<T, U>) -> bool {
        self.edges().all(|e| e.is_left(point))
    }

    /// Does this convex polygon properly contain the given point?
    ///
    /// # Example
    ///
    /// ```
    /// use euclid::point2;
    /// use fart_2d_geom::ConvexPolygon;
    ///
    /// let p = ConvexPolygon::<i32, ()>::hull(vec![
    ///     point2(0, 0),
    ///     point2(10, 2),
    ///     point2(5, 10),
    /// ]).unwrap();
    ///
    /// assert!(p.improperly_contains_point(point2(5, 5)));
    /// assert!(!p.improperly_contains_point(point2(-3, -3)));
    ///
    /// // Points exactly on the edge are considered contained.
    /// assert!(p.improperly_contains_point(point2(0, 0)));
    /// ```
    pub fn improperly_contains_point(&self, point: Point2D<T, U>) -> bool {
        self.edges().all(|e| e.is_left_or_collinear(point))
    }
}

impl<T, U> ConvexPolygon<T, U>
where
    T: Copy + NumCast,
{
    /// Cast from number representation `T` to number representation `V`.
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::ConvexPolygon;
    ///
    /// let hull_f64 = ConvexPolygon::<f64, UnknownUnit>::hull(vec![
    ///     point2(0.0, 0.0),
    ///     point2(2.0, -1.0),
    ///     point2(1.0, 1.0),
    /// ]).expect("non-collinear vertices have a convex hull");
    ///
    /// let hull_i64 = hull_f64.cast::<i64>();
    /// # let _ = hull_i64;
    /// ```
    #[inline]
    pub fn cast<V>(&self) -> ConvexPolygon<V, U>
    where
        V: NumCast + Copy,
    {
        ConvexPolygon {
            inner: self.inner.cast(),
        }
    }
}

impl<T, U> ConvexPolygon<T, U>
where
    T: Copy + Num + PartialOrd + euclid::Trig,
{
    /// Transform this convex polygon with the given linear transformation and
    /// return the new, transformed convex polygon.
    ///
    /// ```
    /// use euclid::{point2, Angle, Transform2D, UnknownUnit};
    /// use fart_2d_geom::ConvexPolygon;
    ///
    /// let hull = ConvexPolygon::<f64, UnknownUnit>::hull(vec![
    ///     point2(0.0, 0.0),
    ///     point2(1.0, 1.0),
    ///     point2(2.0, -1.0),
    /// ]).expect("should have a convex hull for non-collinear vertex sets");
    ///
    /// let rotation = Transform2D::<_, _, UnknownUnit>::create_rotation(Angle::degrees(60.0));
    ///
    /// let rotated_hull = hull.transform(&rotation);
    /// # let _ = rotated_hull;
    /// ```
    pub fn transform<V>(
        &self,
        transformation: &euclid::Transform2D<T, U, V>,
    ) -> ConvexPolygon<T, V> {
        ConvexPolygon {
            inner: self.inner.transform(transformation),
        }
    }

    /// Transform this convex polygon in place with the given linear
    /// transformation.
    ///
    /// ```
    /// use euclid::{point2, Angle, Transform2D, UnknownUnit};
    /// use fart_2d_geom::ConvexPolygon;
    ///
    /// let mut hull = ConvexPolygon::<f64, UnknownUnit>::hull(vec![
    ///     point2(0.0, 0.0),
    ///     point2(1.0, 1.0),
    ///     point2(2.0, -1.0),
    /// ]).expect("should have a convex hull for non-collinear vertex sets");
    ///
    /// let rotation = Transform2D::create_rotation(Angle::degrees(60.0));
    ///
    /// hull.transform_in_place(&rotation);
    /// ```
    pub fn transform_in_place(&mut self, transformation: &euclid::Transform2D<T, U, U>) {
        self.inner.transform_in_place(transformation);
    }
}

impl<T, U> ToAabb<T, U> for ConvexPolygon<T, U>
where
    T: Copy + Num + PartialOrd,
{
    fn to_aabb(&self) -> Aabb<T, U> {
        self.inner.to_aabb()
    }
}
