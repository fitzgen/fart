use euclid::{point2, TypedPoint2D, UnknownUnit};
use fart_2d_geom::ConvexPolygon;
use quickcheck::quickcheck;

fn check_all_vertices_within_convex_hull(vertices: Vec<(i64, i64)>) -> bool {
    let vertices = vertices
        .into_iter()
        .map(|(a, b)| point2(a, b))
        .collect::<Vec<TypedPoint2D<i64, UnknownUnit>>>();

    let p = match ConvexPolygon::hull(vertices.clone()) {
        None => return true,
        Some(p) => p,
    };

    vertices.into_iter().all(|v| p.improperly_contains_point(v))
}

quickcheck! {
    fn all_vertices_within_convex_hull(vertices: Vec<(i64, i64)>) -> bool {
        check_all_vertices_within_convex_hull(vertices)
    }
}

#[test]
fn all_zeros() {
    assert!(check_all_vertices_within_convex_hull(vec![
        (0,0),
        (0,0),
        (0,0),
        (0,0),
    ]));
}

#[test]
fn all_collinear() {
    assert!(check_all_vertices_within_convex_hull(vec![
        (1,1),
        (2,2),
        (3,3),
        (4,4),
    ]));
}

#[test]
fn regression0() {
    assert!(check_all_vertices_within_convex_hull(vec![
        (0, 0),
        (0, 1),
        (1, 0)
    ]));
}

#[test]
fn regression1() {
    assert!(check_all_vertices_within_convex_hull(vec![
        (0, 0),
        (0, 0),
        (0, -1),
        (1, 0),
        (1, 1)
    ]));
}

#[test]
fn regression2() {
    assert!(check_all_vertices_within_convex_hull(vec![
        (0, 0),
        (0, -1),
        (-1, 0),
        (0, 1)
    ]));
}

#[test]
fn regression3() {
    assert!(check_all_vertices_within_convex_hull(vec![
        (0, 0),
        (0, 1),
        (0, 2)
    ]));
}

#[test]
fn regression4() {
    assert!(check_all_vertices_within_convex_hull(vec![
        (9, 5),
        (0, -4),
        (0, 0),
        (-1, 0),
        (8, 4)
    ]));
}
