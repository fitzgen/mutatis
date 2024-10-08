//! A thin-but-stable wrapper over `rand::rngs::SmallRng` that provides a few
//! more conveniences for our use-cases.

use core::sync::atomic::{self, AtomicU32};

use rand::{rngs::SmallRng, Rng as _, SeedableRng};

/// A pseudorandom number generator.
///
/// Not cryptographically secure.
///
/// You can attain a reference to an `Rng` via the
/// [`Context::rng`][crate::Context::rng] method.
#[derive(Clone, Debug)]
pub struct Rng {
    inner: SmallRng,
}

impl Default for Rng {
    fn default() -> Self {
        static DEFAULT_SEED: AtomicU32 = AtomicU32::new(0);
        Self::new(DEFAULT_SEED.fetch_add(1, atomic::Ordering::Relaxed).into())
    }
}

macro_rules! gen_methods {
    ( $( $name:ident -> $ty:ty ; )* ) => {
        $(
            /// Generate a random
            #[doc = concat!("`", stringify!($ty), "`")]
            /// value.
            pub fn $name(&mut self) -> $ty {
                self.inner.gen()
            }
        )*
    };
}

impl Rng {
    #[inline]
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            inner: SmallRng::seed_from_u64(seed),
        }
    }

    #[inline]
    pub(crate) fn inner(&mut self) -> &mut SmallRng {
        &mut self.inner
    }

    /// Generate a random `usize` in the range `0..len`.
    ///
    /// If `len` is `0`, then `None` is returned.
    #[inline]
    pub fn gen_index(&mut self, len: usize) -> Option<usize> {
        if len == 0 {
            return None;
        }

        Some(self.inner.gen_range(0..len))
    }

    /// Choose a random element from an iterator.
    ///
    /// If the iterator is empty, then `None` is returned.
    #[inline]
    pub fn choose<I>(&mut self, iter: I) -> Option<I::Item>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        let idx = self.gen_index(iter.len())?;
        iter.into_iter().nth(idx)
    }

    /// Generate random bytes to fill the provided `buf`.
    #[inline]
    pub fn gen_bytes(&mut self, buf: &mut [u8]) {
        self.inner.fill(buf);
    }

    gen_methods! {
        gen_char -> char;
        gen_bool -> bool;
        gen_u8 -> u8;
        gen_u16 -> u16;
        gen_u32 -> u32;
        gen_u64 -> u64;
        gen_u128 -> u128;
        gen_usize -> usize;
        gen_i8 -> i8;
        gen_i16 -> i16;
        gen_i32 -> i32;
        gen_i64 -> i64;
        gen_i128 -> i128;
        gen_isize -> isize;
        gen_f32 -> f32;
        gen_f64 -> f64;
    }
}
