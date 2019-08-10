use crate::{area2, is_counter_clockwise, line, Line};
use euclid::{point2, Point2D};
use fart_aabb::{Aabb, ToAabb};
use fart_utils::NoMorePartial;
use num_traits::{Num, NumAssign, NumCast, Signed};
use rand::prelude::*;
use std::collections::{BTreeSet, HashSet};
use std::fmt;

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
    vertices: Vec<Point2D<T, U>>,
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
    pub fn new(vertices: Vec<Point2D<T, U>>) -> Polygon<T, U> {
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
            vertices: &[Point2D<T, U>],
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
    pub fn vertices(&self) -> &[Point2D<T, U>] {
        &self.vertices
    }

    /// Get the `i`<sup>th</sup> point in this polygon.
    pub fn get(&self, i: usize) -> Option<Point2D<T, U>> {
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
        F: FnMut(Point2D<T, U>, Point2D<T, U>, Point2D<T, U>),
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

impl<T, U> Polygon<T, U>
where
    T: Copy + NumCast,
{
    /// Cast from number representation `T` to number representation `V`.
    ///
    /// ```
    /// use euclid::{point2, UnknownUnit};
    /// use fart_2d_geom::Polygon;
    ///
    /// let tri_f64 = Polygon::<f64, UnknownUnit>::new(vec![
    ///     point2(0.0, 0.0),
    ///     point2(2.0, -1.0),
    ///     point2(1.0, 1.0),
    /// ]);
    ///
    /// let tri_i64 = tri_f64.cast::<i64>();
    /// # let _ = tri_i64;
    /// ```
    pub fn cast<V>(&self) -> Polygon<V, U>
    where
        V: NumCast + Copy,
    {
        Polygon {
            vertices: self.vertices.iter().map(|p| p.cast()).collect(),
        }
    }
}

impl<T, U> Polygon<T, U>
where
    T: Copy + Num + PartialOrd + euclid::Trig,
{
    /// Transform this polygon with the given linear transformation and return
    /// the new, transformed polygon.
    ///
    /// ```
    /// use euclid::{point2, Angle, Transform2D, UnknownUnit};
    /// use fart_2d_geom::Polygon;
    ///
    /// let tri = Polygon::<f64, UnknownUnit>::new(vec![
    ///     point2(0.0, 0.0),
    ///     point2(2.0, -1.0),
    ///     point2(1.0, 1.0),
    /// ]);
    ///
    /// let rotation = Transform2D::<_, _, UnknownUnit>::create_rotation(Angle::degrees(60.0));
    ///
    /// let rotated_tri = tri.transform(&rotation);
    /// # let _ = rotated_tri;
    /// ```
    pub fn transform<V>(&self, transformation: &euclid::Transform2D<T, U, V>) -> Polygon<T, V> {
        Polygon {
            vertices: self
                .vertices
                .iter()
                .cloned()
                .map(|p| transformation.transform_point(p))
                .collect(),
        }
    }

    /// Transform this polygon in place with the given linear transformation.
    ///
    /// ```
    /// use euclid::{point2, Angle, Transform2D, UnknownUnit};
    /// use fart_2d_geom::Polygon;
    ///
    /// let mut tri = Polygon::<_, UnknownUnit>::new(vec![
    ///     point2(0.0, 0.0),
    ///     point2(2.0, -1.0),
    ///     point2(1.0, 1.0),
    /// ]);
    ///
    /// let rotation = Transform2D::create_rotation(Angle::degrees(60.0));
    ///
    /// tri.transform_in_place(&rotation);
    /// ```
    pub fn transform_in_place(&mut self, transformation: &euclid::Transform2D<T, U, U>) {
        for p in &mut self.vertices {
            *p = transformation.transform_point(*p);
        }
    }
}

impl<T, U> ToAabb<T, U> for Polygon<T, U>
where
    T: Copy + Num + PartialOrd,
{
    fn to_aabb(&self) -> Aabb<T, U> {
        Aabb::for_vertices(self.vertices.iter().cloned())
    }
}
