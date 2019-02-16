use euclid_aabb::{Aabb, AabbTree};
use quickcheck::{quickcheck, Arbitrary};

// Until https://github.com/servo/euclid/pull/318 merges...
#[derive(Clone, Debug, Eq, PartialEq)]
struct U;

type P2<T> = euclid::TypedPoint2D<T, U>;

fn p2<T>(a: T, b: T) -> P2<T> {
    P2::new(a, b)
}

#[derive(Clone, Debug)]
struct ArbitraryAabb(Aabb<f64, U>);

impl Arbitrary for ArbitraryAabb {
    fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> ArbitraryAabb {
        let xs = [f64::arbitrary(g), f64::arbitrary(g)];
        let ys = [f64::arbitrary(g), f64::arbitrary(g)];
        ArbitraryAabb(Aabb::new(
            p2(f64::min(xs[0], xs[1]), f64::min(ys[0], ys[1])),
            p2(f64::max(xs[0], xs[1]), f64::max(ys[0], ys[1])),
        ))
    }
}

quickcheck! {
    fn contains_boxes(boxes: Vec<ArbitraryAabb>) -> bool {
        let mut tree = AabbTree::new();
        for b in boxes.iter() {
            tree.insert(b.0.clone(), ());
        }
        boxes.into_iter().all(|b| tree.any_overlap(b.0))
    }
}

#[test]
fn intersects() {
    let a = Aabb::new(p2(-1.0, -1.0), p2(1.0, 1.0));
    assert!(a.intersects(&a));

    for b in vec![
        // Shifted to the side, but overlapping.
        Aabb::new(p2(-2.0, -1.0), p2(0.0, 1.0)),
        Aabb::new(p2(-1.0, -2.0), p2(1.0, 0.0)),
        Aabb::new(p2(0.0, -1.0), p2(2.0, 1.0)),
        Aabb::new(p2(-1.0, 0.0), p2(1.0, 2.0)),
        // Contained.
        Aabb::new(p2(-0.5, -0.5), p2(0.5, 0.5)),
        // Contains.
        Aabb::new(p2(-10.0, -10.0), p2(10.0, 10.0)),
    ] {
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    for c in vec![
        // Shifted outside.
        Aabb::new(p2(-4.0, -1.0), p2(-2.0, 1.0)),
        Aabb::new(p2(-1.0, -4.0), p2(1.0, -2.0)),
        Aabb::new(p2(2.0, -1.0), p2(4.0, 1.0)),
        Aabb::new(p2(-1.0, 2.0), p2(1.0, 4.0)),
        // Touching edges, but not overlapping.
        Aabb::new(p2(-3.0, -1.0), p2(-1.0, 1.0)),
        Aabb::new(p2(-1.0, -3.0), p2(1.0, -1.0)),
        Aabb::new(p2(1.0, -1.0), p2(3.0, 1.0)),
        Aabb::new(p2(-1.0, 1.0), p2(1.0, 3.0)),
    ] {
        assert!(!a.intersects(&c));
        assert!(!c.intersects(&a));
    }
}

#[test]
fn join() {
    let a = Aabb::new(p2(-1.0, -1.0), p2(1.0, 1.0));
    assert_eq!(a.join(&a), a);

    for b in vec![
        // Shifted to the side, but overlapping.
        Aabb::new(p2(-2.0, -1.0), p2(0.0, 1.0)),
        Aabb::new(p2(-1.0, -2.0), p2(1.0, 0.0)),
        Aabb::new(p2(0.0, -1.0), p2(2.0, 1.0)),
        Aabb::new(p2(-1.0, 0.0), p2(1.0, 2.0)),
        // Contained.
        Aabb::new(p2(-0.5, -0.5), p2(0.5, 0.5)),
        // Contains.
        Aabb::new(p2(-10.0, -10.0), p2(10.0, 10.0)),
        // Shifted outside.
        Aabb::new(p2(-4.0, -1.0), p2(-2.0, 1.0)),
        Aabb::new(p2(-1.0, -4.0), p2(1.0, -2.0)),
        Aabb::new(p2(2.0, -1.0), p2(4.0, 1.0)),
        Aabb::new(p2(-1.0, 2.0), p2(1.0, 4.0)),
        // Touching edges, but not overlapping.
        Aabb::new(p2(-3.0, -1.0), p2(-1.0, 1.0)),
        Aabb::new(p2(-1.0, -3.0), p2(1.0, -1.0)),
        Aabb::new(p2(1.0, -1.0), p2(3.0, 1.0)),
        Aabb::new(p2(-1.0, 1.0), p2(1.0, 3.0)),
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
    let a = Aabb::new(p2(-1.0, -1.0), p2(1.0, 1.0));
    assert!(a.contains(&a));

    for b in vec![
        // All edges strictly contained.
        Aabb::new(p2(-0.5, -0.5), p2(0.5, 0.5)),
        // One edge strictly contained, other same.
        Aabb::new(p2(-0.5, -1.0), p2(1.0, 1.0)),
        Aabb::new(p2(-1.0, -0.5), p2(1.0, 1.0)),
        Aabb::new(p2(-1.0, -1.0), p2(0.5, 1.0)),
        Aabb::new(p2(-1.0, -1.0), p2(1.0, 0.5)),
    ] {
        assert!(a.contains(&b));
        assert!(b.contains(&b));
    }

    for c in vec![
        // Shifted to the side, and non-overlapping.
        Aabb::new(p2(-4.0, -1.0), p2(-2.0, 1.0)),
        Aabb::new(p2(-1.0, -4.0), p2(1.0, 2.0)),
        Aabb::new(p2(2.0, -1.0), p2(4.0, 1.0)),
        Aabb::new(p2(-1.0, 2.0), p2(1.0, 4.0)),
        // Shifted to the side, but overlapping.
        Aabb::new(p2(-2.0, -1.0), p2(0.0, 1.0)),
        Aabb::new(p2(-1.0, -2.0), p2(1.0, 0.0)),
        Aabb::new(p2(0.0, -1.0), p2(2.0, 1.0)),
        Aabb::new(p2(-1.0, 0.0), p2(1.0, 2.0)),
        // Contains a.
        Aabb::new(p2(-10.0, -10.0), p2(10.0, 10.0)),
    ] {
        assert!(!a.contains(&c));
        assert!(c.contains(&c));
    }
}

#[test]
fn tree() {
    let mut tree = AabbTree::new();

    let alice_aabb = Aabb::new(p2(0.0, 0.0), p2(2.0, 2.0));
    tree.insert(alice_aabb.clone(), "Alice");
    let bob_aabb = Aabb::new(p2(2.0, 2.0), p2(4.0, 4.0));
    tree.insert(bob_aabb.clone(), "Bob");
    let zed_aabb = Aabb::new(p2(10.0, 10.0), p2(20.0, 20.0));
    tree.insert(zed_aabb.clone(), "Zed");

    for (target, expected) in vec![
        (
            Aabb::new(p2(-100.0, -100.0), p2(100.0, 100.0)),
            vec![
                (alice_aabb.clone(), "Alice"),
                (bob_aabb.clone(), "Bob"),
                (zed_aabb.clone(), "Zed"),
            ],
        ),
        (
            Aabb::new(p2(1.0, 1.0), p2(3.0, 3.0)),
            vec![(alice_aabb.clone(), "Alice"), (bob_aabb.clone(), "Bob")],
        ),
        (Aabb::new(p2(100.0, 100.0), p2(300.0, 300.0)), vec![]),
        (
            Aabb::new(p2(0.0, 0.0), p2(1.0, 1.0)),
            vec![(alice_aabb.clone(), "Alice")],
        ),
        (
            Aabb::new(p2(2.0, 2.0), p2(3.0, 3.0)),
            vec![(bob_aabb.clone(), "Bob")],
        ),
        (
            Aabb::new(p2(10.0, 10.0), p2(15.0, 15.0)),
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
