use euclid::{point2, TypedPoint2D};
use fart_2d_geom::Polygon;
use quickcheck::{quickcheck, Arbitrary, Gen};
use rand::distributions::{Distribution, Uniform};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct UnknownUnit;

#[derive(Clone, Debug)]
struct ArbitraryPolygon(Polygon<i64, UnknownUnit>);

impl Arbitrary for ArbitraryPolygon {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let n = Uniform::new(3, 10).sample(g);
        ArbitraryPolygon(Polygon::random(
            g,
            &mut Uniform::new(-100, 100),
            &mut Uniform::new(-100, 100),
            n,
        ))
    }

    // TODO: support shrinking. Needs to handle cases where removing a vertex
    // transforms a simple polygon into a complex polygon, and when removing a
    // vertex causes the clockwise vs counter-clockwise sorting to be flipped.

    // fn shrink(&self) -> Box<Iterator<Item = Self>> {
    //     let vertices = self.0.vertices().iter().cloned().collect::<Vec<_>>();
    //     let smaller = if vertices.iter().all(|v| v.x % 2 == 0 && v.y % 2 == 0) {
    //         Some(ArbitraryPolygon(Polygon::new(
    //             vertices
    //                 .iter()
    //                 .map(|v| point2(v.x / 2, v.y / 2))
    //                 .collect::<Vec<_>>(),
    //         )))
    //     } else {
    //         None
    //     };
    //     Box::new(
    //         smaller
    //             .into_iter()
    //             .chain((0..self.0.len()).flat_map(move |i| {
    //                 if vertices.len() > 3 {
    //                     let mut vs = vertices.clone();
    //                     vs.remove(i);
    //                     Some(ArbitraryPolygon(Polygon::new(vs)))
    //                 } else {
    //                     None
    //                 }
    //             })),
    //     )
    // }
}

fn check_triangulation_uses_every_vertex(polygon: Polygon<i64, UnknownUnit>) -> bool {
    let vertices = polygon
        .vertices()
        .iter()
        .cloned()
        .collect::<HashSet<TypedPoint2D<i64, UnknownUnit>>>();

    let mut yet_to_see = vertices.clone();
    let mut saw_unknown = false;

    polygon.triangulate(|a, b, c| {
        saw_unknown |= !vertices.contains(&a);
        saw_unknown |= !vertices.contains(&b);
        saw_unknown |= !vertices.contains(&c);
        yet_to_see.remove(&a);
        yet_to_see.remove(&b);
        yet_to_see.remove(&c);
    });

    yet_to_see.is_empty() && !saw_unknown
}

fn check_triangulation_area_is_polygon_area(polygon: Polygon<i64, UnknownUnit>) -> bool {
    let polygon_area = polygon.area();
    let mut trianglulation_area = 0;
    let mut potential_error = 0;
    polygon.triangulate(|a, b, c| {
        println!("triangle: {:?} {:?} {:?}", a, b, c);
        potential_error += 1;
        let triangle = Polygon::new(vec![a, b, c]);
        trianglulation_area += triangle.area();
    });
    let delta = (polygon_area - trianglulation_area).abs();
    // Allow for rounding errors.
    delta < potential_error
}

fn check_all(p: Polygon<i64, UnknownUnit>) -> bool {
    check_triangulation_area_is_polygon_area(p.clone()) && check_triangulation_uses_every_vertex(p)
}

quickcheck! {
    fn triangulation_uses_every_vertex(polygon: ArbitraryPolygon) -> bool {
        check_triangulation_uses_every_vertex(polygon.0)
    }

    fn triangulation_area_is_polygon_area(polygon: ArbitraryPolygon) -> bool {
        check_triangulation_area_is_polygon_area(polygon.0)
    }
}

#[test]
fn some_collinear_vertices() {
    check_all(Polygon::new(vec![
        point2(2, 0),
        point2(1, 0),
        point2(0, 0),
        point2(0, -1),
    ]));
}

#[test]
fn all_collinear_vertices() {
    check_all(Polygon::new(vec![
        point2(0, 0),
        point2(1, 0),
        point2(2, 0),
        point2(3, 0),
    ]));
}

#[test]
fn rectangle() {
    check_all(Polygon::new(vec![
        point2(-2, -2),
        point2(1, -2),
        point2(1, 1),
        point2(-2, 1),
    ]));
}

#[test]
fn convex_pentagon() {
    check_all(Polygon::new(vec![
        point2(-2, -2),
        point2(2, -2),
        point2(3, 1),
        point2(0, 3),
        point2(-3, 1),
    ]));
}

#[test]
fn non_convex_pentagon() {
    check_all(Polygon::new(vec![
        point2(-2, -2),
        point2(2, -2),
        point2(0, 0),
        point2(2, 2),
        point2(-2, 2),
    ]));
}

#[test]
fn regression0() {
    check_all(Polygon::new(vec![
        point2(2, -1),
        point2(1, 2),
        point2(0, -2),
        point2(1, -2),
    ]));
}
