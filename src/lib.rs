//! # `fart`: **f**itzgen's generative **art**
//!
//! ## Example
//!
//! Drawing a random triangle!
//!
//! ```no_run
//! use fart::rand::distributions::{Distribution, Uniform};
//! use fart::{aabb, scene, shape, Point2};
//!
//! fn main() {
//!     fart::generate(|cfg| {
//!         let mut scene = scene::Scene::new(aabb::AxisAlignedBoundingBox::new(
//!             Point2::new(0.0, 0.0),
//!             Point2::new(1000.0, 1000.0),
//!         ));
//!
//!         let x_dist = Uniform::new(0.0, 1000.0);
//!         let y_dist = Uniform::new(0.0, 1000.0);
//!
//!         scene.add(shape::Triangle {
//!             a: Point2::new(x_dist.sample(cfg.rng()), y_dist.sample(cfg.rng())),
//!             b: Point2::new(x_dist.sample(cfg.rng()), y_dist.sample(cfg.rng())),
//!             c: Point2::new(x_dist.sample(cfg.rng()), y_dist.sample(cfg.rng())),
//!         });
//!
//!         Ok(scene.create_svg(scene::Inches(7.0), scene::Inches(7.0)))
//!     });
//! }
//! ```

#![deny(missing_docs, missing_debug_implementations)]

pub mod aabb;
pub mod path;
pub mod scene;
pub mod shape;

// Re-exports of our public dependencies.
pub use euclid;
pub use failure;
pub use rand;
pub use svg;

use failure::ResultExt;
use rand::SeedableRng;
use std::env;
use std::path::PathBuf;
use std::process;
use std::str;

/// Either an `Ok(T)` or an `Err(failure::Error)`.
pub type Result<T> = ::std::result::Result<T, failure::Error>;

/// A two dimensional point.
pub type Point2 = euclid::Point2D<f64>;

/// A two dimensional vector.
pub type Vector2 = euclid::Vector2D<f64>;

/// Configuration options for SVG generation.
#[derive(Debug)]
pub struct Config {
    file_name: PathBuf,
    rng: rand::rngs::SmallRng,
}

impl Config {
    fn new() -> Result<Config> {
        let file_name =
            env::var("FART_FILE_NAME").context("missing required FART_FILE_NAME env var")?;
        let file_name = PathBuf::from(file_name);

        let seed = if let Ok(seed) = env::var("FART_RNG_SEED") {
            str::parse::<u64>(&seed)
                .context("failed to parse the FART_RNG_SEED env var as a u64")?
        } else {
            rand::random()
        };
        let rng = rand::rngs::SmallRng::seed_from_u64(seed);

        Ok(Config { file_name, rng })
    }

    /// Get a reference to the random number generator.
    ///
    /// All random code should use this rng so that if we seed it with the same
    /// value, we ultimately get the same image.
    #[inline]
    pub fn rng(&mut self) -> &mut impl rand::Rng {
        &mut self.rng
    }
}

/// Clamp a value to within some range.
///
/// # Example
///
/// ```
/// use fart::clamp;
///
/// let x = clamp(5.0, 0.0, 10.0);
/// assert_eq!(x, 5.0);
///
/// let y = clamp(11.0, 0.0, 10.0);
/// assert_eq!(y, 10.0);
///
/// let z = clamp(-5.0, 0.0, 10.0);
/// assert_eq!(z, 0.0);
/// ```
///
/// # Panics
///
/// Panics if `low > high`.
pub fn clamp(value: f64, low: f64, high: f64) -> f64 {
    assert!(low <= high);
    match value {
        x if x < low => low,
        x if x > high => high,
        x => x,
    }
}

/// Map a value from one range to another range.
///
/// # Example
///
/// ```
/// use fart::map_range;
///
/// let x = map_range(0.5, 0.0, 1.0, 0.0, 10.0);
/// assert_eq!(x, 5.0);
///
/// let y = map_range(3.0, 2.0, 5.0, 0.0, 3.0);
/// assert_eq!(y, 1.0);
/// ```
///
/// # Panics
///
/// Panics if the given value is outside the input range, if `in_low > in_high`,
/// or if `out_low > out_high`.
pub fn map_range(value: f64, in_low: f64, in_high: f64, out_low: f64, out_high: f64) -> f64 {
    assert!(in_low <= in_high);
    assert!(out_low <= out_high);
    assert!(value >= in_low);
    assert!(value <= in_high);
    let slope = 1.0 * (out_high - out_low) / (in_high - in_low);
    out_low + (slope * (value - in_low)).round()
}

/// Generate an SVG with the given function `f`.
///
/// ```no_run
/// # #![allow(warnings)]
/// extern crate fart;
///
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
    let code = match try_generate(f) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {}", e);
            for c in e.iter_causes() {
                eprintln!("    Caused by: {}", c);
            }
            1
        }
    };
    process::exit(code);
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
