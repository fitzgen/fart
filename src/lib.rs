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
//!         let mut scene = Scene::new(Aabb::new(
//!             point2(0, 0),
//!             point2(1000, 1000),
//!         ));
//!
//!         let x_dist = Uniform::new(0, 1000);
//!         let y_dist = Uniform::new(0, 1000);
//!
//!         scene.add(Triangle {
//!             a: point2(x_dist.sample(cfg.rng()), y_dist.sample(cfg.rng())),
//!             b: point2(x_dist.sample(cfg.rng()), y_dist.sample(cfg.rng())),
//!             c: point2(x_dist.sample(cfg.rng()), y_dist.sample(cfg.rng())),
//!         });
//!
//!         Ok(scene.create_svg(Inches(7.0), Inches(7.0)))
//!     });
//! }
//! ```

#![deny(missing_docs, missing_debug_implementations)]

pub mod path;
pub mod prelude;
pub mod scene;
pub mod shape;

// Re-exports of our public dependencies.
pub use euclid;
pub use euclid_aabb as aabb;
pub use failure;
pub use num_traits;
pub use rand;
pub use svg;

use failure::ResultExt;
use num_traits::{Num, NumCast};
use rand::SeedableRng;
use std::env;
use std::ops::{Range, RangeInclusive};
use std::path::PathBuf;
use std::process;
use std::str;

/// Either an `Ok(T)` or an `Err(failure::Error)`.
pub type Result<T> = ::std::result::Result<T, failure::Error>;

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
/// let x = clamp(5.0, 0.0..=10.0);
/// assert_eq!(x, 5.0);
///
/// let y = clamp(11.0, 0.0..=10.0);
/// assert_eq!(y, 10.0);
///
/// let z = clamp(-5.0, 0.0..=10.0);
/// assert_eq!(z, 0.0);
/// ```
///
/// # Panics
///
/// Panics if `range.start() > range.end()`.
pub fn clamp<N>(value: N, range: RangeInclusive<N>) -> N
where
    N: PartialOrd,
{
    let (low, high) = range.into_inner();
    assert!(low <= high);
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

/// Map a value from one range to another range.
///
/// # Example
///
/// ```
/// use fart::map_range;
///
/// let x = map_range(5, 0..10, 0..100);
/// assert_eq!(x, 50);
///
/// let y = map_range(3, 2..5, 0..3);
/// assert_eq!(y, 1);
/// ```
///
/// # Panics
///
/// Panics if the given value is outside the input range, if `in_low >= in_high`,
/// or if `out_low >= out_high`, or if number conversions fail.
pub fn map_range<N, M>(
    value: N,
    Range {
        start: in_low,
        end: in_high,
    }: Range<N>,
    Range {
        start: out_low,
        end: out_high,
    }: Range<M>,
) -> M
where
    N: Num + NumCast + Copy + PartialOrd,
    M: Num + NumCast + Copy + PartialOrd,
{
    assert!(in_low < in_high);
    assert!(out_low < out_high);
    assert!(value >= in_low);
    assert!(value <= in_high);

    let value: M = NumCast::from(value).unwrap();
    let in_low: M = NumCast::from(in_low).unwrap();
    let in_high: M = NumCast::from(in_high).unwrap();

    let dividend = out_high - out_low;
    let divisor = in_high - in_low;
    assert!(!divisor.is_zero());

    let slope = dividend / divisor;
    out_low + (slope * (value - in_low))
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
