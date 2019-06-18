//! A canvas for drawing paths on.

use crate::aabb::Aabb;
use crate::path::{Path, ToPaths};

/// Unit for things within the canvas space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasSpace;

/// A canvas is a collection of rendered paths. To add new paths to the canvas,
/// use the `draw` method.
#[derive(Debug)]
pub struct Canvas {
    view: Aabb<i64, CanvasSpace>,
    paths: Vec<Path<i64, CanvasSpace>>,
    stroke_width: i64,
}

impl Canvas {
    /// Construct a new canvas with the given viewport.
    pub fn new(view: Aabb<i64, CanvasSpace>) -> Canvas {
        let stroke_width = view.width() / 500;
        Canvas {
            view,
            paths: Vec::new(),
            stroke_width,
        }
    }

    /// Get the stroke width for paths in this canvas.
    pub fn stroke_width(&self) -> i64 {
        self.stroke_width
    }

    /// Set the stroke width for paths in this canvas.
    pub fn set_stroke_width(&mut self, stroke_width: i64) {
        self.stroke_width = stroke_width;
    }

    /// Get this canvas's view.
    #[inline]
    pub fn view(&self) -> &Aabb<i64, CanvasSpace> {
        &self.view
    }

    /// Add the given paths to the canvas.
    pub fn draw<P>(&mut self, paths: &P)
    where
        P: ToPaths<i64, CanvasSpace>,
    {
        self.paths.extend(paths.to_paths());
    }

    /// Render this canvas as an SVG with the given physical width and height.
    ///
    /// # Example
    ///
    /// Make a 3" x 3" SVG from a canvas.
    ///
    /// ```
    /// use fart::aabb::Aabb;
    /// use fart::euclid::point2;
    /// use fart::canvas::{Inches, Canvas};
    ///
    /// let canvas = Canvas::new(Aabb::new(
    ///     point2(0, 0),
    ///     point2(100, 100),
    /// ));
    /// let svg_doc = canvas.create_svg(Inches(3.0), Inches(3.0));
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

/// Express an canvas's SVG's physical dimensions in inches.
///
/// See `Canvas::create_svg` for examples.
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

/// Express an canvas's SVG's physical dimensions in millimeters.
///
/// See `Canvas::create_svg` for examples.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Millis(pub f64);

impl From<Millis> for f64 {
    fn from(i: Millis) -> f64 {
        i.0
    }
}

impl SvgUnit for Millis {
    const SUFFIX: &'static str = "mm";
}
