#[macro_use]
extern crate quickcheck;

use fart::{
    aabb::{AabbTree, AxisAlignedBoundingBox},
    Point2,
};

#[derive(Clone, Debug)]
struct Aabb(AxisAlignedBoundingBox);

impl quickcheck::Arbitrary for Aabb {
    fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Aabb {
        let xs = [f64::arbitrary(g), f64::arbitrary(g)];
        let ys = [f64::arbitrary(g), f64::arbitrary(g)];
        Aabb(AxisAlignedBoundingBox::new(
            Point2::new(f64::min(xs[0], xs[1]), f64::min(ys[0], ys[1])),
            Point2::new(f64::max(xs[0], xs[1]), f64::max(ys[0], ys[1])),
        ))
    }
}

quickcheck! {
    fn contains_boxes(boxes: Vec<Aabb>) -> bool {
        let mut tree = AabbTree::new();
        for b in boxes.iter() {
            tree.insert(b.0.clone(), ());
        }
        boxes.into_iter().all(|b| tree.any_overlap(b.0))
    }
}

#[test]
fn intersects() {
    let a = AxisAlignedBoundingBox::new(Point2::new(-1.0, -1.0), Point2::new(1.0, 1.0));
    assert!(a.intersects(&a));

    for b in vec![
        // Shifted to the side, but overlapping.
        AxisAlignedBoundingBox::new(Point2::new(-2.0, -1.0), Point2::new(0.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, -2.0), Point2::new(1.0, 0.0)),
        AxisAlignedBoundingBox::new(Point2::new(0.0, -1.0), Point2::new(2.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, 0.0), Point2::new(1.0, 2.0)),
        // Contained.
        AxisAlignedBoundingBox::new(Point2::new(-0.5, -0.5), Point2::new(0.5, 0.5)),
        // Contains.
        AxisAlignedBoundingBox::new(Point2::new(-10.0, -10.0), Point2::new(10.0, 10.0)),
    ] {
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    for c in vec![
        // Shifted outside.
        AxisAlignedBoundingBox::new(Point2::new(-4.0, -1.0), Point2::new(-2.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, -4.0), Point2::new(1.0, -2.0)),
        AxisAlignedBoundingBox::new(Point2::new(2.0, -1.0), Point2::new(4.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, 2.0), Point2::new(1.0, 4.0)),
        // Touching edges, but not overlapping.
        AxisAlignedBoundingBox::new(Point2::new(-3.0, -1.0), Point2::new(-1.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, -3.0), Point2::new(1.0, -1.0)),
        AxisAlignedBoundingBox::new(Point2::new(1.0, -1.0), Point2::new(3.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, 1.0), Point2::new(1.0, 3.0)),
    ] {
        assert!(!a.intersects(&c));
        assert!(!c.intersects(&a));
    }
}

#[test]
fn join() {
    let a = AxisAlignedBoundingBox::new(Point2::new(-1.0, -1.0), Point2::new(1.0, 1.0));
    assert_eq!(a.join(&a), a);

    for b in vec![
        // Shifted to the side, but overlapping.
        AxisAlignedBoundingBox::new(Point2::new(-2.0, -1.0), Point2::new(0.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, -2.0), Point2::new(1.0, 0.0)),
        AxisAlignedBoundingBox::new(Point2::new(0.0, -1.0), Point2::new(2.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, 0.0), Point2::new(1.0, 2.0)),
        // Contained.
        AxisAlignedBoundingBox::new(Point2::new(-0.5, -0.5), Point2::new(0.5, 0.5)),
        // Contains.
        AxisAlignedBoundingBox::new(Point2::new(-10.0, -10.0), Point2::new(10.0, 10.0)),
        // Shifted outside.
        AxisAlignedBoundingBox::new(Point2::new(-4.0, -1.0), Point2::new(-2.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, -4.0), Point2::new(1.0, -2.0)),
        AxisAlignedBoundingBox::new(Point2::new(2.0, -1.0), Point2::new(4.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, 2.0), Point2::new(1.0, 4.0)),
        // Touching edges, but not overlapping.
        AxisAlignedBoundingBox::new(Point2::new(-3.0, -1.0), Point2::new(-1.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, -3.0), Point2::new(1.0, -1.0)),
        AxisAlignedBoundingBox::new(Point2::new(1.0, -1.0), Point2::new(3.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, 1.0), Point2::new(1.0, 3.0)),
    ] {
        assert_eq!(b.join(&b), b);
        let j = a.join(&b);
        assert!(j.contains(&a));
        assert!(j.intersects(&a));
        assert!(a.intersects(&j));
        assert!(j.contains(&b));
        assert!(j.intersects(&b));
        assert!(b.intersects(&j));
    }
}

#[test]
fn contains() {
    let a = AxisAlignedBoundingBox::new(Point2::new(-1.0, -1.0), Point2::new(1.0, 1.0));
    assert!(a.contains(&a));

    for b in vec![
        // All edges strictly contained.
        AxisAlignedBoundingBox::new(Point2::new(-0.5, -0.5), Point2::new(0.5, 0.5)),
        // One edge strictly contained, other same.
        AxisAlignedBoundingBox::new(Point2::new(-0.5, -1.0), Point2::new(1.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, -0.5), Point2::new(1.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, -1.0), Point2::new(0.5, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, -1.0), Point2::new(1.0, 0.5)),
    ] {
        assert!(a.contains(&b));
        assert!(b.contains(&b));
    }

    for c in vec![
        // Shifted to the side, and non-overlapping.
        AxisAlignedBoundingBox::new(Point2::new(-4.0, -1.0), Point2::new(-2.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, -4.0), Point2::new(1.0, 2.0)),
        AxisAlignedBoundingBox::new(Point2::new(2.0, -1.0), Point2::new(4.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, 2.0), Point2::new(1.0, 4.0)),
        // Shifted to the side, but overlapping.
        AxisAlignedBoundingBox::new(Point2::new(-2.0, -1.0), Point2::new(0.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, -2.0), Point2::new(1.0, 0.0)),
        AxisAlignedBoundingBox::new(Point2::new(0.0, -1.0), Point2::new(2.0, 1.0)),
        AxisAlignedBoundingBox::new(Point2::new(-1.0, 0.0), Point2::new(1.0, 2.0)),
        // Contains a.
        AxisAlignedBoundingBox::new(Point2::new(-10.0, -10.0), Point2::new(10.0, 10.0)),
    ] {
        assert!(!a.contains(&c));
        assert!(c.contains(&c));
    }
}

#[test]
fn tree() {
    let mut tree = AabbTree::new();

    let alice_aabb = AxisAlignedBoundingBox::new(Point2::new(0.0, 0.0), Point2::new(2.0, 2.0));
    tree.insert(alice_aabb.clone(), "Alice");
    let bob_aabb = AxisAlignedBoundingBox::new(Point2::new(2.0, 2.0), Point2::new(4.0, 4.0));
    tree.insert(bob_aabb.clone(), "Bob");
    let zed_aabb = AxisAlignedBoundingBox::new(Point2::new(10.0, 10.0), Point2::new(20.0, 20.0));
    tree.insert(zed_aabb.clone(), "Zed");

    for (target, expected) in vec![
        (
            AxisAlignedBoundingBox::new(Point2::new(-100.0, -100.0), Point2::new(100.0, 100.0)),
            vec![
                (alice_aabb.clone(), "Alice"),
                (bob_aabb.clone(), "Bob"),
                (zed_aabb.clone(), "Zed"),
            ],
        ),
        (
            AxisAlignedBoundingBox::new(Point2::new(1.0, 1.0), Point2::new(3.0, 3.0)),
            vec![(alice_aabb.clone(), "Alice"), (bob_aabb.clone(), "Bob")],
        ),
        (
            AxisAlignedBoundingBox::new(Point2::new(100.0, 100.0), Point2::new(300.0, 300.0)),
            vec![],
        ),
        (
            AxisAlignedBoundingBox::new(Point2::new(0.0, 0.0), Point2::new(1.0, 1.0)),
            vec![(alice_aabb.clone(), "Alice")],
        ),
        (
            AxisAlignedBoundingBox::new(Point2::new(2.0, 2.0), Point2::new(3.0, 3.0)),
            vec![(bob_aabb.clone(), "Bob")],
        ),
        (
            AxisAlignedBoundingBox::new(Point2::new(10.0, 10.0), Point2::new(15.0, 15.0)),
            vec![(zed_aabb.clone(), "Zed")],
        ),
    ] {
        let mut overlaps: Vec<_> = tree
            .iter_overlapping(target)
            .map(|(aabb, who)| (aabb.clone(), *who))
            .collect();
        overlaps.sort_by_key(|&(_, w)| w);
        assert_eq!(overlaps, expected);
    }
}
