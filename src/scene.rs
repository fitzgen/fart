//! A scene of shapes.

use crate::aabb::Aabb;
use crate::path::{Path, ToPaths};

/// Unit for things within the scene space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneSpace;

/// A scene is a collection of rendered paths.
#[derive(Debug)]
pub struct Scene {
    view: Aabb<i64, SceneSpace>,
    paths: Vec<Path<i64, SceneSpace>>,
    stroke_width: i64,
}

impl Scene {
    /// Construct a new scene with the given viewport.
    pub fn new(view: Aabb<i64, SceneSpace>) -> Scene {
        let stroke_width = view.width() / 500;
        Scene {
            view,
            paths: Vec::new(),
            stroke_width,
        }
    }

    /// Get the stroke width for paths in this scene.
    pub fn stroke_width(&self) -> i64 {
        self.stroke_width
    }

    /// Set the stroke width for paths in this scene.
    pub fn set_stroke_width(&mut self, stroke_width: i64) {
        self.stroke_width = stroke_width;
    }

    /// Get this scene's view.
    #[inline]
    pub fn view(&self) -> &Aabb<i64, SceneSpace> {
        &self.view
    }

    /// Add the given paths to the scene.
    pub fn draw<P>(&mut self, paths: &P)
    where
        P: ToPaths<i64, SceneSpace>,
    {
        self.paths.extend(paths.to_paths());
    }

    /// Render this scene as an SVG with the given physical width and height.
    ///
    /// # Example
    ///
    /// Make a 3" x 3" SVG from a scene.
    ///
    /// ```
    /// use fart::aabb::Aabb;
    /// use fart::euclid::point2;
    /// use fart::scene::{Inches, Scene};
    ///
    /// let scene = Scene::new(Aabb::new(
    ///     point2(0, 0),
    ///     point2(100, 100),
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
            doc = doc.add(path.set("stroke-width", self.stroke_width));
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
