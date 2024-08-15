//! A thin-but-stable wrapper over `rand::rngs::SmallRng` that provides a few
//! more conveniences for our use-cases.

use rand::{rngs::SmallRng, Rng as _, SeedableRng};

const DEFAULT_SEED: u64 = 0x12345678_12345678;

/// A pseudorandom number generator.
///
/// Not cryptographically secure.
///
/// You can attain a reference to an `Rng` via the
/// [`MutationContext::rng`][crate::MutationContext::rng] method.
#[derive(Clone, Debug)]
pub struct Rng {
    inner: SmallRng,
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(DEFAULT_SEED)
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
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            inner: SmallRng::seed_from_u64(seed),
        }
    }

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

        // https://lemire.me/blog/2016/06/30/fast-random-shuffling/
        let random32bit = u64::from(self.gen_u32());
        let multiresult = random32bit.wrapping_mul(u64::try_from(len).unwrap());
        Some((multiresult >> 32) as usize)
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
