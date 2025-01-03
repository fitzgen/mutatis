//! The provided set of [`Mutate`] implementations and combinators.
//!
//! It is idiomatic to import this module with the alias `m`:
//!
//! ```rust
//! use mutatis::mutators as m;
//! ```

use super::*;
use ::core::{marker::PhantomData, ops};
use rand::Rng;

mod combinators;
mod core_impls;

pub use combinators::*;
pub use core_impls::*;

#[cfg(feature = "alloc")]
mod alloc_impls;
#[cfg(feature = "alloc")]
pub use alloc_impls::*;

// TODO: mod std;
// TODO: pub use std::*;

/// A mutator that doesn't do anything.
///
/// See the [`nop`] function to create new `Nop` mutator instances and for
/// example usage.
#[derive(Clone, Debug)]
pub struct Nop<T> {
    _phantom: PhantomData<fn(&mut T)>,
}

/// Create a mutator that doesn't do anything.
///
/// This can be useful as the initial mutator in a mutator-combinator chain.
///
/// # Example
///
/// ```
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::nop::<(u32, u32)>()
///     .map(|ctx, (a, b)| {
///         let x = ctx.rng().gen_u32();
///         let y = ctx.rng().gen_u32();
///         *a = x.min(y);
///         *b = x.max(y);
///         Ok(())
///     });
/// let mut session = Session::new();
///
/// let mut value = (0, 0);
/// session.mutate_with(&mut mutator, &mut value).unwrap();
///
/// assert!(value.0 <= value.1);
/// ```
pub fn nop<T>() -> Nop<T> {
    Nop {
        _phantom: PhantomData,
    }
}

impl<T> Mutate<T> for Nop<T> {
    fn mutate(&mut self, c: &mut Candidates<'_>, _value: &mut T) -> Result<()> {
        c.mutation(|_| Ok(()))
    }
}

/// A mutator constructed from a function.
pub struct FromFn<F, T> {
    func: F,
    _phantom: PhantomData<fn(&mut T)>,
}

/// Create a mutator from a function.
///
/// The function is given a [`Context`] and an `&mut T` value, and must return a
/// [`mutatis::Result<()>`].
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Context, Mutate, Session};
///
/// let mut mutator = m::from_fn(|ctx: &mut Context, pair: &mut (u32, u32)| {
///     pair.0 = ctx.rng().gen_u32();
///     pair.1 = if ctx.rng().gen_bool() {
///         pair.0.wrapping_add(1)
///     } else {
///         pair.0.wrapping_sub(1)
///     };
///     Ok(())
/// });
/// let mut session = Session::new();
///
/// let mut value = (0, 0);
///
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("{value:?}");
/// }
///
/// // Example output:
///  //
/// //     (1886093101, 1886093102)
/// //     (3852925062, 3852925063)
/// //     (1697131274, 1697131275)
/// //     (4193528377, 4193528378)
/// //     (3958122412, 3958122411)

/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
pub fn from_fn<F, T>(func: F) -> FromFn<F, T> {
    FromFn {
        func,
        _phantom: PhantomData,
    }
}

impl<F, T> Mutate<T> for FromFn<F, T>
where
    F: FnMut(&mut Context, &mut T) -> Result<()>,
{
    fn mutate(&mut self, c: &mut Candidates<'_>, value: &mut T) -> Result<()> {
        c.mutation(|ctx| (self.func)(ctx, value))
    }
}

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
