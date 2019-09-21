//! Incrementally computed processs for drawing onto a canvas.

use crate::canvas::Canvas;

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
///     fn update(&mut self, canvas: &Canvas) -> bool {
///         let mut i = 0;
///
///         let x_dist = Uniform::new(0, canvas.view().width());
///         let y_dist = Uniform::new(0, canvas.view().height());
///
///         let mut rng = fart::rng();
///
///         loop {
///             let xa = x_dist.sample(&mut rng);
///             let xb = x_dist.sample(&mut rng);
///             let xmin = min(xa, xb);
///             let xmax = max(xa, xb);
///
///             let ya = y_dist.sample(&mut rng);
///             let yb = y_dist.sample(&mut rng);
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
///     fn draw(&self, canvas: &mut Canvas, last_frame: bool) {
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
    /// Update the process's state.
    ///
    /// If the process is complete, return `true`. Then there will be a final
    /// `draw` call, and `update` will never be called again.
    ///
    /// If the process is not finished, return `false` and `update` will be
    /// called again in the future.
    fn update(&mut self, canvas: &Canvas) -> bool;

    /// Draw the current state of the process to the given canvas.
    ///
    /// If `last_frame` is true, then this is the last time that `draw` will be
    /// called.
    fn draw(&self, canvas: &mut Canvas, last_frame: bool);
}

/// Run a process to completion, drawing it to the given canvas.
pub fn run<P>(canvas: &mut Canvas, process: &mut P)
where
    P: Process,
{
    loop {
        let last_frame = process.update(canvas);
        process.draw(canvas, last_frame);
        if last_frame {
            break;
        }
    }
}
