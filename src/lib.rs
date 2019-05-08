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
pub use failure;
pub use fart_2d_geom as geom;
pub use fart_aabb as aabb;
pub use noise;
pub use num_traits;
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
