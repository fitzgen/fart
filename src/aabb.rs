//! Axis-aligned bounding boxes (AABB) and AABB trees.
//!
//! Used for fast-but-imprecise collision detection of shapes in a scene. Once
//! candidates for collision are quickly found using an AABB tree, we (will
//! eventually) determine if they precisely collide with a more expensive
//! algorithm.

use crate::Point2;
use std::f64;

/// An axis-aligned bounding box.
#[derive(Debug, Clone, PartialEq)]
pub struct AxisAlignedBoundingBox {
    min: Point2,
    max: Point2,
    area: f64,
}

impl AxisAlignedBoundingBox {
    /// Construct a new axis-aligned bounding box.
    ///
    /// `min`'s `x` and `y` components must be less than or equal to `max`'s.
    #[inline]
    pub fn new(min: Point2, max: Point2) -> AxisAlignedBoundingBox {
        debug_assert!(min.x <= max.x);
        debug_assert!(min.y <= max.y);
        let area = (max.x - min.x) * (max.y - min.y);
        AxisAlignedBoundingBox { min, max, area }
    }

    /// Construct a new axis-aligned bounding box that contains the given set of
    /// vertices.
    ///
    /// # Panics
    ///
    /// Panics if `vertices` is empty.
    pub fn for_vertices<I>(vertices: I) -> AxisAlignedBoundingBox
    where
        I: IntoIterator<Item = Point2>,
    {
        let mut vertices = vertices.into_iter();
        let first = vertices
            .next()
            .expect("Must have at least one vertex to create a bounding box");
        let mut min = first;
        let mut max = first;
        for v in vertices {
            min.x = f64::min(min.x, v.x);
            min.y = f64::min(min.y, v.y);
            max.x = f64::max(max.x, v.x);
            max.y = f64::max(max.y, v.y);
        }
        AxisAlignedBoundingBox::new(min, max)
    }

    /// Get this AABB's min.
    #[inline]
    pub fn min(&self) -> Point2 {
        self.min
    }

    /// Get this AABB's max.
    #[inline]
    pub fn max(&self) -> Point2 {
        self.max
    }

    /// Get this AABB's area.
    #[inline]
    pub fn area(&self) -> f64 {
        self.area
    }

    /// Return the least upper bound of `self` and `other`.
    #[inline]
    pub fn join(&self, other: &AxisAlignedBoundingBox) -> AxisAlignedBoundingBox {
        let min = euclid::Point2D::new(
            f64::min(self.min.x, other.min.x),
            f64::min(self.min.y, other.min.y),
        );
        let max = euclid::Point2D::new(
            f64::max(self.max.x, other.max.x),
            f64::max(self.max.y, other.max.y),
        );
        AxisAlignedBoundingBox::new(min, max)
    }

    /// Does `self` contain `other`?
    pub fn contains(&self, other: &AxisAlignedBoundingBox) -> bool {
        other.min.x >= self.min.x
            && other.max.x <= self.max.x
            && other.min.y >= self.min.y
            && other.max.y <= self.max.y
    }

    /// Does `self` intersect with `other`?
    pub fn intersects(&self, other: &AxisAlignedBoundingBox) -> bool {
        self.max.x > other.min.x
            && self.min.x < other.max.x
            && self.max.y > other.min.y
            && self.min.y < other.max.y
    }
}

/// A tree mapping from axis-aligned bounding boxes to `T` values.
#[derive(Debug)]
pub struct AabbTree<T> {
    root: Option<AabbTreeNode<T>>,
}

#[derive(Debug)]
enum AabbTreeNode<T> {
    Branch(AabbTreeBranch<T>),
    Leaf(AabbTreeLeaf<T>),
}

#[derive(Debug)]
struct AabbTreeBranch<T> {
    aabb: AxisAlignedBoundingBox,
    children: Box<(AabbTreeNode<T>, AabbTreeNode<T>)>,
}

#[derive(Debug)]
struct AabbTreeLeaf<T> {
    aabb: AxisAlignedBoundingBox,
    value: T,
}

impl<T> AabbTree<T> {
    /// Construct a new, empty AABB tree.
    #[inline]
    pub fn new() -> AabbTree<T> {
        AabbTree { root: None }
    }

    /// Insert the given value into the AABB tree.
    pub fn insert(&mut self, aabb: AxisAlignedBoundingBox, value: T) {
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
    /// use fart::{
    ///     Point2,
    ///     aabb::{AabbTree, AxisAlignedBoundingBox},
    /// };
    ///
    /// let mut tree = AabbTree::new();
    /// tree.insert(AxisAlignedBoundingBox::new(Point2::new(0.0, 0.0), Point2::new(2.0, 2.0)), "Alice");
    /// tree.insert(AxisAlignedBoundingBox::new(Point2::new(2.0, 2.0), Point2::new(4.0, 4.0)), "Bob");
    /// tree.insert(AxisAlignedBoundingBox::new(Point2::new(10.0, 10.0), Point2::new(20.0, 20.0)), "Zed");
    ///
    /// let target = AxisAlignedBoundingBox::new(Point2::new(1.0, 1.0), Point2::new(3.0, 3.0));
    /// for (aabb, who) in tree.iter_overlapping(target) {
    ///     match *who {
    ///         "Alice" => println!("Found Alice at {:?}", aabb),
    ///         "Bob" => println!("Found Bob at {:?}", aabb),
    ///         someone => panic!("Found someone we shouldn't have: {}", someone),
    ///     }
    /// }
    /// ```
    pub fn iter_overlapping(&self, aabb: AxisAlignedBoundingBox) -> IterOverlapping<T> {
        let stack = self
            .root
            .iter()
            .filter(|n| n.aabb().intersects(&aabb))
            .collect();
        IterOverlapping { aabb, stack }
    }

    /// Do any of the AABBs in this tree overlap with the give AABB?
    #[inline]
    pub fn any_overlap(&self, aabb: AxisAlignedBoundingBox) -> bool {
        self.iter_overlapping(aabb).next().is_some()
    }
}

impl<T> AabbTreeNode<T> {
    fn aabb(&self) -> &AxisAlignedBoundingBox {
        match self {
            AabbTreeNode::Leaf(l) => &l.aabb,
            AabbTreeNode::Branch(b) => &b.aabb,
        }
    }

    fn insert(self, leaf: AabbTreeLeaf<T>) -> AabbTreeNode<T> {
        match self {
            AabbTreeNode::Leaf(l) => AabbTreeNode::Branch(AabbTreeBranch {
                aabb: l.aabb.join(&leaf.aabb),
                children: Box::new((AabbTreeNode::Leaf(l), AabbTreeNode::Leaf(leaf))),
            }),
            AabbTreeNode::Branch(branch) => {
                let combined_aabb = branch.aabb.join(&leaf.aabb);
                let new_parent_cost = 2.0 * combined_aabb.area;
                let min_push_down_cost = 2.0 * (combined_aabb.area - branch.aabb.area);

                let left_cost = match branch.children.0 {
                    AabbTreeNode::Leaf(ref l) => l.aabb.join(&leaf.aabb).area + min_push_down_cost,
                    AabbTreeNode::Branch(ref b) => {
                        b.aabb.join(&leaf.aabb).area - b.aabb.area + min_push_down_cost
                    }
                };

                let right_cost = match branch.children.1 {
                    AabbTreeNode::Leaf(ref l) => l.aabb.join(&leaf.aabb).area + min_push_down_cost,
                    AabbTreeNode::Branch(ref b) => {
                        b.aabb.join(&leaf.aabb).area - b.aabb.area + min_push_down_cost
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
pub struct IterOverlapping<'a, T> {
    aabb: AxisAlignedBoundingBox,
    stack: Vec<&'a AabbTreeNode<T>>,
}

impl<'a, T> Iterator for IterOverlapping<'a, T> {
    type Item = (&'a AxisAlignedBoundingBox, &'a T);

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
