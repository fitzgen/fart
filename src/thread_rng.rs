use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::{cell::RefCell, marker::PhantomData};

crate::user_const! {
    const RNG_SEED: u64 = 69_420;
}

thread_local! {
    static THREAD_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::seed_from_u64(*RNG_SEED));
}

/// An RNG that is seeded with a `fart::user_const!`.
///
/// `FartThreadRng` is not share-able across threads (not `Send` or
/// `Sync`). Every thread has its own `FartThreadRng` and they are all seeded
/// with the same value.
#[derive(Clone, Copy, Debug, Default)]
pub struct FartThreadRng {
    pub(crate) no_send: PhantomData<*mut ()>,
}

impl RngCore for FartThreadRng {
    fn next_u32(&mut self) -> u32 {
        THREAD_RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            rng.next_u32()
        })
    }

    fn next_u64(&mut self) -> u64 {
        THREAD_RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            rng.next_u64()
        })
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        THREAD_RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            rng.fill_bytes(dest)
        })
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> std::result::Result<(), rand::Error> {
        THREAD_RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            rng.try_fill_bytes(dest)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::FartThreadRng;

    #[test]
    fn fart_thread_rng_impls_rng() {
        // Just making sure that the blanket `impl Rng for R: RngCore` impl
        // keeps existing in the future, or at least I know about it if it
        // changes.
        fn impls_rng(_rng: impl rand::Rng) {}
        impls_rng(FartThreadRng::default());
    }
}
