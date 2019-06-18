//! Two dimensional axis-aligned bounding boxes (AABB) and AABB trees.
//!
//! Used for fast-but-imprecise collision detection of shapes in a scene. Once
//! candidates for collision are quickly found using an AABB tree, can determine
//! if they precisely collide with a more expensive algorithm.

use euclid::TypedPoint2D;
use num_traits::Num;
use partial_min_max::{max as partial_max, min as partial_min};
use std::fmt;

/// An axis-aligned bounding box.
///
/// * `T` is the numeric type. `i32` or `f64` etc.
/// * `U` is the unit. `ScreenSpace` or `WorldSpace` etc.
#[derive(Clone, PartialEq)]
pub struct Aabb<T, U = euclid::UnknownUnit> {
    min: TypedPoint2D<T, U>,
    max: TypedPoint2D<T, U>,
}

impl<T, U> fmt::Debug for Aabb<T, U>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Aabb")
            .field("min", &self.min)
            .field("max", &self.max)
            .finish()
    }
}

impl<T, U> Aabb<T, U>
where
    T: Copy + Num + PartialOrd,
{
    /// Construct a new axis-aligned bounding box.
    ///
    /// `min`'s `x` and `y` components must be less than or equal to `max`'s.
    #[inline]
    pub fn new(min: TypedPoint2D<T, U>, max: TypedPoint2D<T, U>) -> Aabb<T, U> {
        assert!(min.x <= max.x);
        assert!(min.y <= max.y);
        Aabb { min, max }
    }

    /// Construct a new axis-aligned bounding box that contains the given set of
    /// vertices.
    ///
    /// # Panics
    ///
    /// Panics if `vertices` is empty.
    pub fn for_vertices<I>(vertices: I) -> Aabb<T, U>
    where
        I: IntoIterator<Item = TypedPoint2D<T, U>>,
    {
        let mut vertices = vertices.into_iter();
        let first = vertices
            .next()
            .expect("Must have at least one vertex to create a bounding box");
        let mut min = first;
        let mut max = first;
        for v in vertices {
            min.x = partial_min(min.x, v.x);
            min.y = partial_min(min.y, v.y);
            max.x = partial_max(max.x, v.x);
            max.y = partial_max(max.y, v.y);
        }
        Aabb::new(min, max)
    }

    /// Get this AABB's min.
    #[inline]
    pub fn min(&self) -> TypedPoint2D<T, U> {
        self.min
    }

    /// Get this AABB's max.
    #[inline]
    pub fn max(&self) -> TypedPoint2D<T, U> {
        self.max
    }

    /// Get the width of this AABB.
    #[inline]
    pub fn width(&self) -> T {
        self.max.x - self.min.x
    }

    /// Get the height of this AABB.
    #[inline]
    pub fn height(&self) -> T {
        self.max.y - self.min.y
    }

    /// Get this AABB's area.
    #[inline]
    pub fn area(&self) -> T {
        (self.max.x - self.min.x) * (self.max.y - self.min.y)
    }

    /// Return the least upper bound of `self` and `other`.
    #[inline]
    pub fn join(&self, other: &Aabb<T, U>) -> Aabb<T, U> {
        let min = TypedPoint2D::new(
            partial_min(self.min.x, other.min.x),
            partial_min(self.min.y, other.min.y),
        );
        let max = TypedPoint2D::new(
            partial_max(self.max.x, other.max.x),
            partial_max(self.max.y, other.max.y),
        );
        Aabb::new(min, max)
    }

    /// Does `self` contain `other`?
    pub fn contains(&self, other: &Aabb<T, U>) -> bool {
        other.min.x >= self.min.x
            && other.max.x <= self.max.x
            && other.min.y >= self.min.y
            && other.max.y <= self.max.y
    }

    /// Does `self` intersect with `other`?
    pub fn intersects(&self, other: &Aabb<T, U>) -> bool {
        self.max.x > other.min.x
            && self.min.x < other.max.x
            && self.max.y > other.min.y
            && self.min.y < other.max.y
    }
}

/// A tree mapping from axis-aligned bounding boxes to `T` values.
#[derive(Debug, Default)]
pub struct AabbTree<T, U, V> {
    root: Option<AabbTreeNode<T, U, V>>,
}

#[derive(Debug)]
enum AabbTreeNode<T, U, V> {
    Branch(AabbTreeBranch<T, U, V>),
    Leaf(AabbTreeLeaf<T, U, V>),
}

#[derive(Debug)]
struct AabbTreeBranch<T, U, V> {
    aabb: Aabb<T, U>,
    children: Box<(AabbTreeNode<T, U, V>, AabbTreeNode<T, U, V>)>,
}

#[derive(Debug)]
struct AabbTreeLeaf<T, U, V> {
    aabb: Aabb<T, U>,
    value: V,
}

impl<T, U, V> AabbTree<T, U, V>
where
    T: Copy + Num + PartialOrd,
{
    /// Construct a new, empty AABB tree.
    #[inline]
    pub fn new() -> AabbTree<T, U, V> {
        AabbTree { root: None }
    }

    /// Insert the given value into the AABB tree.
    pub fn insert(&mut self, aabb: Aabb<T, U>, value: V) {
        let leaf = AabbTreeLeaf { aabb, value };
        self.root = Some(if let Some(r) = self.root.take() {
            r.insert(leaf)
        } else {
            AabbTreeNode::Leaf(leaf)
        });
    }

    /// Iterate over each of the AABB keys and associated values that overlap
    /// with the given AABB.
    ///
    /// Order of iteration is not defined.
    ///
    /// ```
    /// use euclid::Point2D;
    /// use fart_aabb::{AabbTree, Aabb};
    ///
    /// let mut tree = AabbTree::new();
    /// tree.insert(Aabb::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 2.0)), "Alice");
    /// tree.insert(Aabb::new(Point2D::new(2.0, 2.0), Point2D::new(4.0, 4.0)), "Bob");
    /// tree.insert(Aabb::new(Point2D::new(10.0, 10.0), Point2D::new(20.0, 20.0)), "Zed");
    ///
    /// let target = Aabb::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 3.0));
    /// for (aabb, who) in tree.iter_overlapping(target) {
    ///     match *who {
    ///         "Alice" => println!("Found Alice at {:?}", aabb),
    ///         "Bob" => println!("Found Bob at {:?}", aabb),
    ///         someone => panic!("Found someone we shouldn't have: {}", someone),
    ///     }
    /// }
    /// ```
    pub fn iter_overlapping(&self, aabb: Aabb<T, U>) -> IterOverlapping<T, U, V> {
        let stack = self
            .root
            .iter()
            .filter(|n| n.aabb().intersects(&aabb))
            .collect();
        IterOverlapping { aabb, stack }
    }

    /// Do any of the AABBs in this tree overlap with the give AABB?
    #[inline]
    pub fn any_overlap(&self, aabb: Aabb<T, U>) -> bool {
        self.iter_overlapping(aabb).next().is_some()
    }
}

impl<T, U, V> AabbTreeNode<T, U, V>
where
    T: Copy + Num + PartialOrd,
{
    fn aabb(&self) -> &Aabb<T, U> {
        match self {
            AabbTreeNode::Leaf(l) => &l.aabb,
            AabbTreeNode::Branch(b) => &b.aabb,
        }
    }

    fn insert(self, leaf: AabbTreeLeaf<T, U, V>) -> AabbTreeNode<T, U, V> {
        match self {
            AabbTreeNode::Leaf(l) => AabbTreeNode::Branch(AabbTreeBranch {
                aabb: l.aabb.join(&leaf.aabb),
                children: Box::new((AabbTreeNode::Leaf(l), AabbTreeNode::Leaf(leaf))),
            }),
            AabbTreeNode::Branch(branch) => {
                let combined_aabb = branch.aabb.join(&leaf.aabb);
                let two = T::one() + T::one();
                let new_parent_cost = two * combined_aabb.area();
                let min_push_down_cost = two * (combined_aabb.area() - branch.aabb.area());

                let left_cost = match branch.children.0 {
                    AabbTreeNode::Leaf(ref l) => {
                        l.aabb.join(&leaf.aabb).area() + min_push_down_cost
                    }
                    AabbTreeNode::Branch(ref b) => {
                        b.aabb.join(&leaf.aabb).area() - b.aabb.area() + min_push_down_cost
                    }
                };

                let right_cost = match branch.children.1 {
                    AabbTreeNode::Leaf(ref l) => {
                        l.aabb.join(&leaf.aabb).area() + min_push_down_cost
                    }
                    AabbTreeNode::Branch(ref b) => {
                        b.aabb.join(&leaf.aabb).area() - b.aabb.area() + min_push_down_cost
                    }
                };

                AabbTreeNode::Branch(AabbTreeBranch {
                    aabb: combined_aabb,
                    children: Box::new(
                        if new_parent_cost < left_cost && new_parent_cost < right_cost {
                            (AabbTreeNode::Leaf(leaf), AabbTreeNode::Branch(branch))
                        } else if left_cost < right_cost {
                            (branch.children.0.insert(leaf), branch.children.1)
                        } else {
                            (branch.children.0, branch.children.1.insert(leaf))
                        },
                    ),
                })
            }
        }
    }
}

/// An iterator over overlapping AABBs and values in an AABB tree.
///
/// See `AabbTree::iter_overlapping`.
#[derive(Debug, Clone)]
pub struct IterOverlapping<'a, T, U, V> {
    aabb: Aabb<T, U>,
    stack: Vec<&'a AabbTreeNode<T, U, V>>,
}

impl<'a, T, U, V> Iterator for IterOverlapping<'a, T, U, V>
where
    T: Copy + Num + PartialOrd,
{
    type Item = (&'a Aabb<T, U>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.pop() {
                None => return None,
                Some(AabbTreeNode::Leaf(l)) => {
                    debug_assert!(l.aabb.intersects(&self.aabb));
                    return Some((&l.aabb, &l.value));
                }
                Some(AabbTreeNode::Branch(b)) => {
                    if self.aabb.intersects(b.children.0.aabb()) {
                        self.stack.push(&b.children.0);
                    }
                    if self.aabb.intersects(b.children.1.aabb()) {
                        self.stack.push(&b.children.1);
                    }
                }
            }
        }
    }
}

/// Things that have an axis-aligned bounding box.
///
/// While we can construct an AABB from anything with vertices, implementations
/// of this trait are intended to be the fastest way to get an AABB for the
/// given `Self` type. For example, we can compute the AABB of a circle
/// geometrically faster than by sampling points along it and constructing the
/// AABB of those sampled points.
pub trait ToAabb<T, U> {
    /// Get the axis-aligned bounding box for `self`.
    fn to_aabb(&self) -> Aabb<T, U>;
}
