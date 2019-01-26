//! A scene of shapes.

use crate::aabb::{AabbTree, AxisAlignedBoundingBox};
use crate::path::Path;
use crate::shape::{Shape, ShapeId};
use id_arena::Arena;

/// A scene is a collection of shapes, their rendered paths, and all the shapes'
/// AABBs.
#[derive(Debug)]
pub struct Scene {
    view: AxisAlignedBoundingBox,
    shapes: Arena<Box<Shape>>,
    paths: Vec<Path>,
    bounding_boxes: AabbTree<ShapeId>,
}

impl Scene {
    /// Construct a new scene with the given viewport.
    pub fn new(view: AxisAlignedBoundingBox) -> Scene {
        Scene {
            view,
            shapes: Arena::new(),
            paths: Vec::new(),
            bounding_boxes: AabbTree::new(),
        }
    }

    /// Get the AABB tree for every shape that has been added to the scene.
    pub fn aabb_tree(&self) -> &AabbTree<ShapeId> {
        &self.bounding_boxes
    }

    /// Add the given shape to the scene.
    ///
    /// If the shape's bounding box does not intersect with the scene's
    /// viewport, then the shape's paths are ignored.
    pub fn add<S: 'static + Shape>(&mut self, shape: S) -> ShapeId {
        let aabb = shape.aabb();
        if self.view.intersects(&aabb) {
            self.paths.extend(shape.paths());
        }
        let id = self.shapes.alloc(Box::new(shape));
        self.bounding_boxes.insert(aabb, id);
        id
    }

    /// Get a reference to a shape that was previously inserted into the scene
    /// with the `add` method.
    #[inline]
    pub fn get(&self, id: ShapeId) -> Option<&Shape> {
        self.shapes.get(id).map(|s| &**s)
    }

    /// Render this scene as an SVG with the given physical width and height.
    ///
    /// # Example
    ///
    /// Make a 3" x 3" SVG from a scene.
    ///
    /// ```
    /// use fart::aabb::AxisAlignedBoundingBox;
    /// use fart::scene::{Inches, Scene};
    /// use fart::{Point2, svg};
    ///
    /// let scene = Scene::new(AxisAlignedBoundingBox::new(
    ///     Point2::new(0.0, 0.0),
    ///     Point2::new(100.0, 100.0),
    /// ));
    /// let svg_doc = scene.create_svg(Inches(3.0), Inches(3.0));
    /// # let _ = svg_doc;
    /// ```
    pub fn create_svg<W, H>(&self, width: W, height: H) -> svg::Document
    where
        W: SvgUnit,
        H: SvgUnit,
    {
        let width = width.into();
        let height = height.into();
        let mut doc = svg::Document::new()
            .set(
                "viewBox",
                format!(
                    "{} {} {} {}",
                    self.view.min().x,
                    self.view.min().y,
                    self.view.max().x,
                    self.view.max().y
                ),
            )
            .set("width", format!("{}{}", width, W::SUFFIX))
            .set("height", format!("{}{}", height, H::SUFFIX));
        for path in &self.paths {
            let path: svg::node::element::Path = path.into();
            doc = doc.add(path);
        }
        doc
    }
}

/// A physical unit supported by SVG (inches, centimeters, etc). Used when
/// plotting an image.
pub trait SvgUnit: Into<f64> {
    /// The unit's string suffix.
    const SUFFIX: &'static str;
}

/// Express an scene's SVG's physical dimensions in inches.
///
/// See `Scene::create_svg` for examples.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Inches(pub f64);

impl From<Inches> for f64 {
    fn from(i: Inches) -> f64 {
        i.0
    }
}

impl SvgUnit for Inches {
    const SUFFIX: &'static str = "in";
}
