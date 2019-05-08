//! Small math utility functions for `fart`.

#![deny(missing_docs)]

use num_traits::{Num, NumCast};
use std::cmp;
use std::fmt::Debug;
use std::ops::{Range, RangeInclusive};

/// Clamp a value to within some range.
///
/// # Example
///
/// ```
/// # use fart_utils::clamp;
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
/// # use fart_utils::map_range;
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
    N: Num + NumCast + Copy + PartialOrd + Debug,
    M: Num + NumCast + Copy + PartialOrd + Debug,
{
    assert!(in_low < in_high, "{:?} < {:?}", in_low, in_high);
    assert!(out_low < out_high, "{:?} < {:?}", out_low, out_high);
    assert!(value >= in_low, "{:?} >= {:?}", value, in_low);
    assert!(value <= in_high, "{:?} <= {:?}", value, in_high);

    let value: M = NumCast::from(value).unwrap();
    let in_low: M = NumCast::from(in_low).unwrap();
    let in_high: M = NumCast::from(in_high).unwrap();

    let dividend = out_high - out_low;
    let divisor = in_high - in_low;
    assert!(!divisor.is_zero());

    let slope = dividend / divisor;
    out_low + (slope * (value - in_low))
}

/// Turn a `T: PartialOrd` and/or `T: PartialEq` into `Ord` and/or `Eq`.
#[derive(Copy, Clone, Debug, Default, PartialOrd, PartialEq, Hash)]
pub struct NoMorePartial<T>(pub T);

impl<T: PartialOrd> cmp::Ord for NoMorePartial<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<T: PartialEq> cmp::Eq for NoMorePartial<T> {}
