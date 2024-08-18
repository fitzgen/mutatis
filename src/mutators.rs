//! The provided set of [`Mutate`] implementations and combinators.
//!
//! It is idiomatic to import this module with the alias `m`:
//!
//! ```rust
//! use mutatis::mutators as m;
//! ```

use super::*;
use ::core::ops;
use rand::Rng;

mod combinators;
mod core_impls;

pub use combinators::*;
pub use core_impls::*;

// TODO: mod alloc;
// TODO: pub use alloc::*;

// TODO: mod std;
// TODO: pub use std::*;

/// A convenience function to get the default mutator for a type.
///
/// This is equivalent to `<T as DefaultMutate>::DefaultMutate::default()` but a
/// little less wordy.
pub fn default<T>() -> <T as DefaultMutate>::DefaultMutate
where
    T: DefaultMutate,
{
    T::DefaultMutate::default()
}

/// A mutator for `T` values within a given range.
///
/// See the [`range`] function to create new `Range` mutator instances and for
/// example usage.
#[derive(Clone, Debug)]
pub struct Range<M, T> {
    mutator: M,
    range: ops::RangeInclusive<T>,
}

/// Create a new mutator for `T` values, keeping them within the given range.
///
/// # Example
///
/// ```
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::range(111..=666);
/// let mut session = Session::new();
///
/// let mut value = 123;
/// session.mutate_with(&mut mutator, &mut value).unwrap();
///
/// assert!(value >= 111);
/// assert!(value <= 666);
/// ```
pub fn range<T>(range: ops::RangeInclusive<T>) -> Range<T::DefaultMutate, T>
where
    T: DefaultMutate,
{
    let mutator = default::<T>();
    Range { mutator, range }
}

/// Like [`range`] but uses the given `mutator` rather than the `T`'s default
/// mutator.
pub fn range_with<M, T>(range: ops::RangeInclusive<T>, mutator: M) -> Range<M, T> {
    Range { mutator, range }
}

impl<M, T> Mutate<T> for Range<M, T>
where
    M: MutateInRange<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut T) -> crate::Result<()> {
        self.mutator.mutate_in_range(c, value, &self.range)
    }
}

impl<M, T> Generate<T> for Range<M, T>
where
    M: Generate<T> + MutateInRange<T>,
{
    #[inline]
    fn generate(&mut self, context: &mut Context) -> crate::Result<T> {
        let mut value = self.mutator.generate(context)?;
        context.mutate_in_range_with(&mut self.mutator, &mut value, &self.range)?;
        Ok(value)
    }
}
