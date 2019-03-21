# `partial-min-max`

Provides `min` and `max` functions that work with `PartialOrd`.

```rust
use partial_min_max::{min, max};
use std::f32::NAN;

// Does what you expect for the easy cases...
assert_eq!(min(0.0, 1.0), 0.0);
assert_eq!(max(0.0, 1.0), 1.0);

// In the case of comparisons with NaN or other partial orderings, returns the
// second value.
assert!(min(0.0, NAN).is_nan());
assert_eq!(min(NAN, 0.0), 0.0);
```
