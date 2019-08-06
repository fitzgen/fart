//! Incrementally computed processs for drawing onto a canvas.

use crate::canvas::Canvas;
use crate::Config;

/// A process is something that is incrementally computed and drawn.
///
/// The `update` method mutates `self` and changes it slightly. Then `draw` is
/// called. Then the process repeats. This continues until `true` is returned by
/// `update`, after which `draw` is called a final time, and the loop finishes.
///
/// This is a useful framework for particle systems, recursive subdivision,
/// simulations, etc...
///
/// ## Example
///
/// This example implements a process that does stochastic rectangle packing.
///
/// ```
/// use fart::{aabb::AabbTree, prelude::*};
/// use std::cmp::{max, min};
///
/// #[derive(Default)]
/// struct RectanglePacking {
///     rects: AabbTree<i64, CanvasSpace, ()>,
/// }
///
/// impl Process for RectanglePacking {
///     fn new(_: &mut fart::Config, _: &Canvas) -> Self {
///         Default::default()
///     }
///
///     fn update(&mut self, cfg: &mut fart::Config, canvas: &Canvas) -> bool {
///         let mut i = 0;
///
///         let x_dist = Uniform::new(0, canvas.view().width());
///         let y_dist = Uniform::new(0, canvas.view().height());
///
///         loop {
///             let xa = x_dist.sample(cfg.rng());
///             let xb = x_dist.sample(cfg.rng());
///             let xmin = min(xa, xb);
///             let xmax = max(xa, xb);
///
///             let ya = y_dist.sample(cfg.rng());
///             let yb = y_dist.sample(cfg.rng());
///             let ymin = min(ya, yb);
///             let ymax = max(ya, yb);
///
///             let rect = Aabb::new(point2(xmin, ymin), point2(xmax, ymax));
///
///             if self.rects.any_overlap(rect.clone()) {
///                 i += 1;
///                 continue;
///             }
///
///             return i > 100;
///         }
///     }
///
///     fn draw(&self, _: &mut fart::Config, canvas: &mut Canvas, last_frame: bool) {
///         if !last_frame {
///             return;
///         }
///
///         for (rect, ()) in self.rects.iter_overlapping(canvas.view().clone()) {
///             canvas.draw(rect);
///         }
///     }
/// }
/// ```
pub trait Process {
    /// Create a new instance of the process.
    fn new(cfg: &mut Config, canvas: &Canvas) -> Self;

    /// Update the process's state.
    ///
    /// If the process is complete, return `true`. Then there will be a final
    /// `draw` call, and `update` will never be called again.
    ///
    /// If the process is not finished, return `false` and `update` will be
    /// called again in the future.
    fn update(&mut self, cfg: &mut Config, canvas: &Canvas) -> bool;

    /// Draw the current state of the process to the given canvas.
    ///
    /// If `last_frame` is true, then this is the last time that `draw` will be
    /// called.
    fn draw(&self, cfg: &mut Config, canvas: &mut Canvas, last_frame: bool);
}

/// Run a process to completion, drawing it to the given canvas.
pub fn run<P>(cfg: &mut Config, canvas: &mut Canvas)
where
    P: Process,
{
    let mut process = P::new(cfg, &canvas);

    loop {
        let last_frame = process.update(cfg, canvas);
        process.draw(cfg, canvas, last_frame);
        if last_frame {
            break;
        }
    }
}
