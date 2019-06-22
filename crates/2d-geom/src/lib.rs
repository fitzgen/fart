//! Two-dimensional geometry algorithms built on top of `euclid`.

#![deny(missing_docs, missing_debug_implementations)]

mod convex_polygon;
mod line;
mod polygon;

pub use crate::{convex_polygon::*, line::*, polygon::*};

use euclid::{point2, TypedPoint2D};
use num_traits::{Num, NumAssign, NumCast, Signed};
use std::cmp::Ordering;

#[inline]
fn area2<T, U>(a: TypedPoint2D<T, U>, b: TypedPoint2D<T, U>, c: TypedPoint2D<T, U>) -> T
where
    T: Copy + Num,
{
    (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)
}

/// Find the center (mean) of a set of points.
///
/// # Panics
///
/// Will panic if the given `points` is empty, or if a `T` cannot be created
/// from `points.len()`.
///
/// # Example
///
/// ```
/// use euclid::{point2, UnknownUnit};
/// use fart_2d_geom::center;
///
/// let c = center::<i32, UnknownUnit>(&[
///     point2(0, 0), point2(2, 0),
///     point2(0, 2), point2(2, 2),
/// ]);
///
/// assert_eq!(c, point2(1, 1));
/// ```
pub fn center<T, U>(points: &[TypedPoint2D<T, U>]) -> TypedPoint2D<T, U>
where
    T: Copy + NumAssign + NumCast,
{
    assert!(!points.is_empty());

    // Find the center (mean) of the points.
    let mut sum_x = 0.0_f64;
    let mut sum_y = 0.0_f64;
    for p in points {
        sum_x += p.x.to_f64().unwrap();
        sum_y += p.y.to_f64().unwrap();
    }
    let n = points.len() as f64;

    let cx = sum_x / n;
    let cy = sum_y / n;

    let (cx, cy) = if T::from(0.1) == T::from(0.9) {
        (T::from(cx.round()).unwrap(), T::from(cy.round()).unwrap())
    } else {
        (T::from(cx).unwrap(), T::from(cy).unwrap())
    };

    point2(cx, cy)
}

/// Sort the given `points` around the given `pivot` point in counter-clockwise
/// order, starting from 12 o'clock.
///
/// # Example
///
/// ```
/// use euclid::{point2, TypedPoint2D, UnknownUnit};
/// use fart_2d_geom::{center, sort_around};
///
/// let mut points: Vec<TypedPoint2D<i32, UnknownUnit>> = vec![
///     point2(0, 2), point2(2, 2),
///     point2(0, 0), point2(2, 0),
/// ];
///
/// let pivot = center(&points);
/// sort_around(pivot, &mut points);
///
/// assert_eq!(points, vec![
///     point2(0, 2),
///     point2(0, 0),
///     point2(2, 0),
///     point2(2, 2),
/// ]);
/// ```
pub fn sort_around<T, U>(pivot: TypedPoint2D<T, U>, points: &mut [TypedPoint2D<T, U>])
where
    T: Copy + NumAssign + PartialOrd + Signed,
{
    points.sort_by(|&a, &b| {
        let zero = T::zero();

        let a_dx = a.x - pivot.x;
        let b_dx = b.x - pivot.x;

        if a_dx >= zero && b_dx < zero {
            Ordering::Greater
        } else if a_dx < zero && b_dx >= zero {
            Ordering::Less
        } else if a_dx == zero && b_dx == zero {
            // Break ties with distance to the pivot.
            if a.y - pivot.y >= zero || b.y - pivot.y >= zero {
                a.y.partial_cmp(&b.y).unwrap()
            } else {
                b.y.partial_cmp(&a.y).unwrap()
            }
        } else {
            let c = (a - pivot).cross(b - pivot);
            if c < zero {
                Ordering::Greater
            } else if c > zero {
                Ordering::Less
            } else {
                // Again, break ties with distance to the pivot.
                let d1 = a.to_vector().cross(pivot.to_vector());
                let d2 = b.to_vector().cross(pivot.to_vector());
                d1.partial_cmp(&d2).unwrap()
            }
        }
    });

    debug_assert!(is_counter_clockwise(points));
}

/// Are the given vertices in counter-clockwise order?
///
/// ```
/// use euclid::{point2, UnknownUnit};
/// use fart_2d_geom::is_counter_clockwise;
///
/// assert!(is_counter_clockwise::<i32, UnknownUnit>(&[
///     point2(0, 1),
///     point2(0, 0),
///     point2(1, 0),
///     point2(1, 1),
/// ]));
///
/// assert!(!is_counter_clockwise::<i32, UnknownUnit>(&[
///     point2(1, 1),
///     point2(1, 0),
///     point2(0, 0),
///     point2(0, 1),
/// ]));
/// ```
pub fn is_counter_clockwise<T, U>(vertices: &[TypedPoint2D<T, U>]) -> bool
where
    T: Copy + NumAssign + Signed + PartialOrd,
{
    let mut sum = T::zero();
    for (i, j) in (0..vertices.len()).zip((1..vertices.len()).chain(Some(0))) {
        let a = vertices[i];
        let b = vertices[j];
        sum += (b.x - a.x) * (b.y + a.y);
    }
    sum <= T::zero()
}
