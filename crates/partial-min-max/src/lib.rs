/// A version of `std::cmp::min` that works with `PartialOrd` types.
///
/// If `a < b` return `a`, otherwise return `b`.
///
/// ```
/// use partial_min_max::min;
/// use std::f32::NAN;
///
/// assert_eq!(min(0.0, 1.0), 0.0);
///
/// assert!(min(0.0, NAN).is_nan());
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
/// ```
/// use partial_min_max::max;
/// use std::f32::NAN;
///
/// assert_eq!(max(0.0, 1.0), 1.0);
///
/// assert!(max(0.0, NAN).is_nan());
/// ```
#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}
