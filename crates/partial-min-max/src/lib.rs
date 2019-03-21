//! `min` and `max` functions that work with `PartialOrd`.
//!
//! When given `NaN`s and other values that don't have total orderings, the
//! functions have well-defined (but arbitrary) behavior: return the second
//! argument.
//!
//! ```
//! use partial_min_max::{min, max};
//! use std::f32::NAN;
//!
//! // Does what you expect for the easy cases...
//! assert_eq!(min(0.0, 1.0), 0.0);
//! assert_eq!(max(0.0, 1.0), 1.0);
//!
//! // In the case of comparisons with NaN or other partial orderings, returns the
//! // second value.
//! assert!(min(0.0, NAN).is_nan());
//! assert_eq!(min(NAN, 0.0), 0.0);
//! ```

/// A version of `std::cmp::min` that works with `PartialOrd` types.
///
/// If `a < b` return `a`, otherwise return `b`.
///
/// # Example
///
/// ```
/// use partial_min_max::min;
/// use std::f32::NAN;
///
/// assert_eq!(min(0.0, 1.0), 0.0);
///
/// assert!(min(0.0, NAN).is_nan());
/// assert_eq!(min(NAN, 0.0), 0.0);
/// ```
#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

/// A version of `std::cmp::max` that works with `PartialOrd` types.
///
/// If `a > b` return `a`, otherwise return `b`.
///
/// # Example
///
/// ```
/// use partial_min_max::max;
/// use std::f32::NAN;
///
/// assert_eq!(max(0.0, 1.0), 1.0);
///
/// assert!(max(0.0, NAN).is_nan());
/// assert_eq!(max(NAN, 0.0), 0.0);
/// ```
#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}
