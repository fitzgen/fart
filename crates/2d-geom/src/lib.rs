//! Two-dimensional geometry algorithms built on top of `euclid`.

#![deny(missing_docs, missing_debug_implementations)]

use euclid::{point2, TypedPoint2D};
use fart_utils::NoMorePartial;
use num_traits::{Bounded, Num, NumAssign, NumCast, Signed};
use partial_min_max::{max, min};
use rand::{distributions::Distribution, seq::IteratorRandom, RngCore};
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::fmt;
use std::ops::{Deref, DerefMut};

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

/// A polygon.
///
/// The polygon's vertices are in counter-clockwise order.
///
/// No guarantees whether this polygon is convex or not.
///
/// * `T` is the numeric type. `i32` or `f64` etc.
/// * `U` is the unit. `ScreenSpace` or `WorldSpace` etc.
#[derive(Clone)]
pub struct Polygon<T, U> {
    vertices: Vec<TypedPoint2D<T, U>>,
}

impl<T, U> fmt::Debug for Polygon<T, U>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Polygon")
            .field(
                "vertices",
                &self
                    .vertices
                    .iter()
                    .map(|v| (&v.x, &v.y))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<T, U> Polygon<T, U>
where
    T: Copy + NumAssign + PartialOrd + Signed + fmt::Debug,
{
    /// Construct a new polygon.
    pub fn new(vertices: Vec<TypedPoint2D<T, U>>) -> Polygon<T, U> {
        assert!(vertices.len() >= 3);
        assert!(
            is_counter_clockwise(&vertices),
            "vertices are not counter clockwise: {:#?}",
            vertices
        );
        Polygon { vertices }
    }

    /// Generate a random `n`-gon with the given `x` an `y` point distributions.
    ///
    /// ```
    /// use euclid::UnknownUnit;
    /// use fart_2d_geom::Polygon;
    /// use rand::{thread_rng, distributions::Uniform};
    ///
    /// // Generate a random pentagon whose vertices are uniformly distributed
    /// // between `(0, 0)` and `(100, 100)`
    /// let pentagon = Polygon::<f64, UnknownUnit>::random(
    ///     &mut thread_rng(),
    ///     &mut Uniform::new(0.0, 100.0),
    ///     &mut Uniform::new(0.0, 100.0),
    ///     5
    /// );
    /// ```
    pub fn random(
        rng: &mut dyn RngCore,
        x_dist: &mut impl Distribution<T>,
        y_dist: &mut impl Distribution<T>,
        n: usize,
    ) -> Polygon<T, U>
    where
        T: NumCast,
    {
        assert!(n >= 3);

        let mut vertices_set = BTreeSet::new();
        let mut point = move |rng: &mut dyn RngCore| {
            for _ in 0..10 {
                let x = x_dist.sample(rng);
                let y = y_dist.sample(rng);
                if vertices_set.insert(NoMorePartial((x, y))) {
                    return point2(x, y);
                }
            }
            panic!("failed to generate a new unique random point with the given distributions")
        };

        let mut vertices = Vec::with_capacity(n);
        vertices.push(point(rng));
        vertices.push(point(rng));
        vertices.push(point(rng));

        // Ensure that the vertices are in counter-clockwise order.
        if !is_counter_clockwise(&vertices) {
            vertices.reverse();
        }
        debug_assert!(is_counter_clockwise(&vertices));

        let mut candidates = HashSet::new();
        for _ in 3..n {
            let v = point(rng);
            for i in 0..vertices.len() {
                let l = line(vertices[(i + vertices.len() - 1) % vertices.len()], v);
                let m = line(v, vertices[i]);
                if !any_edges_collide_with(&vertices, l, m) {
                    candidates.insert(i);
                }
            }
            // Choose one of the candidates for insertion. If there are none,
            // then the vertices are all collinear, and we can insert our new
            // vertex anywhere.
            let i = candidates.drain().choose(rng).unwrap_or(vertices.len());
            vertices.insert(i, v);
            if !is_counter_clockwise(&vertices) {
                vertices.reverse();
                debug_assert!(is_counter_clockwise(&vertices));
            }
        }

        return Polygon::new(vertices);

        fn any_edges_collide_with<T, U>(
            vertices: &[TypedPoint2D<T, U>],
            l: Line<T, U>,
            m: Line<T, U>,
        ) -> bool
        where
            T: Num + Copy + PartialOrd,
        {
            for j in 0..vertices.len() {
                let a = vertices[j];
                let b = vertices[(j + 1 + vertices.len()) % vertices.len()];

                let n = line(a, b);
                if a != l.a && a != l.b && b != l.a && b != l.b && l.improperly_intersects(&n) {
                    return true;
                }
                if a != m.a && a != m.b && b != m.a && b != m.b && m.improperly_intersects(&n) {
                    return true;
                }
            }
            false
        }
    }

    /// Get this polygon's vertices.
    pub fn vertices(&self) -> &[TypedPoint2D<T, U>] {
        &self.vertices
    }

    /// Get the `i`<sup>th</sup> point in this polygon.
    pub fn get(&self, i: usize) -> Option<TypedPoint2D<T, U>> {
        self.vertices.get(i).cloned()
    }

    /// Get the number of vertices in this polygon.
    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    /// Get the index of the next vertex after `i` in this polygon. Handles
    /// wrapping around back to index `0` for you.
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

    /// Get the index of the previous vertex after `i` in this polygon. Handles
    /// wrapping around back to index `n - 1` for you.
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
    /// use fart_2d_geom::Polygon;
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

    /// Do the `a`<sup>th</sup> and `b`<sup>th</sup> vertices within this
    /// polygon form a diagonal?
    ///
    /// If `a` and `b` are diagonal, then there is a direct line of sight
    /// between them, and they are internal to this polygon.
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::Polygon;
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

    /// Is `b` within the cone from `prev(a)` to `a` to `next(a)`?
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::Polygon;
    ///
    /// // For a convex polygon, all non-adjacent points should be in cone.
    /// let p: Polygon<i32, UnknownUnit> = Polygon::new(vec![
    ///     point2(2, -1),
    ///     point2(1, 2),
    ///     point2(0, -2),
    ///     point2(1, -2),
    /// ]);
    ///
    /// assert!(p.in_cone(0, 2));
    /// assert!(p.in_cone(1, 3));
    /// assert!(p.in_cone(2, 0));
    /// assert!(p.in_cone(3, 1));
    ///
    /// // For a non-convex polygon, that is not always the case!
    /// let p: Polygon<i32, UnknownUnit> = Polygon::new(vec![
    ///     point2(0, 0),
    ///     point2(3, -1),
    ///     point2(1, 0),
    ///     point2(3, 1),
    /// ]);
    ///
    /// assert!(p.in_cone(0, 2));
    /// assert!(!p.in_cone(1, 3));
    /// assert!(p.in_cone(2, 0));
    /// assert!(!p.in_cone(3, 1));
    /// ```
    #[inline]
    pub fn in_cone(&self, a: usize, b: usize) -> bool {
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

    /// Triangulate this polygon by ear cutting.
    ///
    /// The given function `f` is invoked with the vertices that make up each
    /// triangle in this polygon's triangulation.
    ///
    /// This is an *O(n<sup>2</sup>)* algorithm.
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::Polygon;
    ///
    /// let p: Polygon<i32, UnknownUnit> = Polygon::new(vec![
    ///     point2(0, 0),
    ///     point2(1, 0),
    ///     point2(1, 1),
    ///     point2(0, 1),
    /// ]);
    ///
    /// p.triangulate(|a, b, c| {
    ///     println!("Triangle within the triangulation: {:?} {:?} {:?}", a, b, c);
    /// });
    /// ```
    pub fn triangulate<F>(mut self, mut f: F)
    where
        F: FnMut(TypedPoint2D<T, U>, TypedPoint2D<T, U>, TypedPoint2D<T, U>),
    {
        // First, process all collinear vertices, since they cause problems with
        // the earcutting algorithm below.
        for i in (0..self.len()).rev() {
            if self.vertices.len() == 3 {
                break;
            }

            if line(self.vertices[self.prev(i)], self.vertices[i])
                .is_collinear(self.vertices[self.next(i)])
            {
                f(
                    self.vertices[self.prev(i)],
                    self.vertices[i],
                    self.vertices[self.next(i)],
                );
                self.vertices.remove(i);
            }
        }

        // `ears[i] == true` if `i` is the tip of an ear.
        let mut ears = (0..self.len())
            .map(|i| self.is_diagonal(self.prev(i), self.next(i)))
            .collect::<Vec<_>>();

        // While we haven't reached the base case....
        while self.vertices.len() > 3 {
            // Search for the next ear.
            let i = ears
                .iter()
                .rposition(|e| *e)
                .expect("if there are no ears, then this is not a simple polygon");

            let prev = self.prev(i);
            let prev_prev = self.prev(prev);
            let next = self.next(i);
            let next_next = self.next(next);

            // Report the ear's triangle.
            f(self.vertices[prev], self.vertices[i], self.vertices[next]);

            // Update the earity of the diagonal's end points.
            ears[prev] = self.is_diagonal(prev_prev, next);
            ears[next] = self.is_diagonal(prev, next_next);

            // Pull the ol' Van Gogh trick!
            ears.remove(i);
            self.vertices.remove(i);
            debug_assert_eq!(ears.len(), self.vertices.len());
        }

        f(self.vertices[0], self.vertices[1], self.vertices[2]);
    }

    /// Iterate over this polygon's edge lines.
    ///
    /// # Example
    ///
    /// ```
    /// use euclid::point2;
    /// use fart_2d_geom::{line, Polygon};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    /// struct WorldSpaceUnits;
    ///
    /// let p = Polygon::<i32, WorldSpaceUnits>::new(vec![
    ///     point2(0, 0),
    ///     point2(0, 2),
    ///     point2(1, 1),
    ///     point2(2, 2),
    ///     point2(0, 2),
    /// ]);
    ///
    /// assert_eq!(
    ///     p.edges().collect::<Vec<_>>(),
    ///     [
    ///         line(point2(0, 0), point2(0, 2)),
    ///         line(point2(0, 2), point2(1, 1)),
    ///         line(point2(1, 1), point2(2, 2)),
    ///         line(point2(2, 2), point2(0, 2)),
    ///         line(point2(0, 2), point2(0, 0)),
    ///     ]
    /// );
    /// ```
    pub fn edges<'a>(&'a self) -> impl 'a + Iterator<Item = Line<T, U>> {
        let ps = self.vertices.iter().cloned();
        let qs = self
            .vertices
            .iter()
            .cloned()
            .skip(1)
            .chain(Some(self.vertices[0]));

        ps.zip(qs).map(|(p, q)| line(p, q))
    }

    /// Get the `idx`th edge.
    ///
    /// This is the edge from vertex `idx` to vertex `idx + 1`.
    pub fn edge(&self, idx: usize) -> Line<T, U> {
        assert!(idx < self.vertices.len());
        let a = self.vertices()[idx];
        let b = self.vertices()[self.next(idx)];
        line(a, b)
    }
}

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

impl<T, U> Deref for ConvexPolygon<T, U> {
    type Target = Polygon<T, U>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, U> DerefMut for ConvexPolygon<T, U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
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
    pub fn hull(mut vertices: Vec<TypedPoint2D<T, U>>) -> Option<ConvexPolygon<T, U>> {
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
    pub fn contains_point(&self, point: TypedPoint2D<T, U>) -> bool {
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
    pub fn improperly_contains_point(&self, point: TypedPoint2D<T, U>) -> bool {
        self.edges().all(|e| e.is_left_or_collinear(point))
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
