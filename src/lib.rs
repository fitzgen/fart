//! # `fart`: **f**itzgen's generative **art**
//!
//! ## Example
//!
//! Drawing a random triangle!
//!
//! ```no_run
//! use fart::prelude::*;
//!
//! fn main() {
//!     fart::generate(|cfg| {
//!         let mut canvas = Canvas::new(Aabb::new(
//!             point2(0, 0),
//!             point2(1000, 1000),
//!         ));
//!
//!         let x_dist = Uniform::new(0, 1000);
//!         let y_dist = Uniform::new(0, 1000);
//!
//!         let triangle = fart::geom::Polygon::new(vec![
//!             point2(x_dist.sample(cfg.rng()), y_dist.sample(cfg.rng())),
//!             point2(x_dist.sample(cfg.rng()), y_dist.sample(cfg.rng())),
//!             point2(x_dist.sample(cfg.rng()), y_dist.sample(cfg.rng())),
//!         ]);
//!
//!         canvas.draw(&triangle);
//!
//!         Ok(canvas.create_svg(Millis(200.0), Millis(200.0)))
//!     });
//! }
//! ```

#![deny(missing_docs, missing_debug_implementations)]

pub mod canvas;
pub mod path;
pub mod prelude;
pub mod process;

mod thread_rng;
mod user_const;

// Re-exports of our public dependencies.
pub use euclid;
pub use failure;
#[doc(inline)]
pub use fart_2d_geom as geom;
#[doc(inline)]
pub use fart_aabb as aabb;
pub use lazy_static;
pub use noise;
pub use num_traits;
pub use rand;
pub use svg;

pub use thread_rng::FartThreadRng;

use failure::ResultExt;
use std::env;
use std::path::PathBuf;

/// Either an `Ok(T)` or an `Err(failure::Error)`.
pub type Result<T> = ::std::result::Result<T, failure::Error>;

/// Configuration options for SVG generation.
#[derive(Debug)]
pub struct Config {
    file_name: PathBuf,
    rng: FartThreadRng,
}

impl Config {
    fn new() -> Result<Config> {
        let file_name =
            env::var("FART_FILE_NAME").context("missing required FART_FILE_NAME env var")?;
        let file_name = PathBuf::from(file_name);

        let rng = rng();

        Ok(Config { file_name, rng })
    }

    /// Get a random number generator.
    ///
    /// Deprecated. Use `fart::rng()` instead.
    #[inline]
    #[deprecated(note = "Use `fart::rng()` instead.")]
    pub fn rng(&mut self) -> &mut impl rand::Rng {
        &mut self.rng
    }
}

/// Generate an SVG with the given function `f`.
///
/// ```no_run
/// # #![allow(warnings)]
/// fn main() {
///     fart::generate(|cfg| {
///         unimplemented!("Your code here...")
///     });
/// }
/// ```
pub fn generate<F>(f: F) -> !
where
    F: FnOnce(&mut Config) -> Result<svg::Document>,
{
    let then = std::time::Instant::now();
    let code = match try_generate(f) {
        Ok(()) => {
            eprintln!(
                "Generated in {:?}",
                std::time::Instant::now().duration_since(then)
            );
            0
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            for c in e.iter_causes() {
                eprintln!("    Caused by: {}", c);
            }
            1
        }
    };
    std::process::exit(code);
}

fn try_generate<F>(f: F) -> Result<()>
where
    F: FnOnce(&mut Config) -> Result<svg::Document>,
{
    let mut config = Config::new().context("failed to read configuration")?;
    let doc = f(&mut config).context("function supplied to `fart::generate` failed")?;
    svg::save(&config.file_name, &doc).context("failed to save SVG to a file")?;
    Ok(())
}

/// Get this thread's `FartThreadRng`!
pub fn rng() -> FartThreadRng {
    return FartThreadRng::default();
}
