use super::*;
use crate::Result;

/// A mutator combinator for applying one of two different mutators.
///
/// See the [`or`][Mutate::or] method on the [`Mutate`] trait for details and
/// example usage.
pub struct Or<M1, M2> {
    pub(crate) left: M1,
    pub(crate) right: M2,
}

impl<M1, M2, T> Mutate<T> for Or<M1, M2>
where
    M1: Mutate<T>,
    M2: Mutate<T>,
{
    fn mutate(&mut self, muts: &mut MutationSet, value: &mut T) -> Result<()> {
        self.left.mutate(muts, value)?;
        self.right.mutate(muts, value)?;
        Ok(())
    }
}

/// A mutator combinator for mapping a function over the mutations produced by
/// another mutator.
///
/// See the [`map`][Mutate::map] method on the [`Mutate`] trait for details and
/// example usage.
#[derive(Clone, Debug)]
pub struct Map<M, F> {
    pub(crate) mutator: M,
    pub(crate) f: F,
}

impl<M, F, T> Mutate<T> for Map<M, F>
where
    M: Mutate<T>,
    F: FnMut(&mut MutationContext, &mut T) -> Result<()>,
{
    fn mutate(&mut self, muts: &mut MutationSet, value: &mut T) -> Result<()> {
        match self.mutator.mutate(muts, value) {
            Err(e) if e.is_early_exit() => {
                (self.f)(&mut muts.context, value)?;
                Err(Error::early_exit())
            }
            res => res,
        }
    }
}

/// A mutator combinator for projecting a value to a sub-value and applying a
/// mutator to that sub-value.
///
/// See the [`proj`][Mutate::proj] method on the [`Mutate`] trait to construct
/// this type, for examples, for more information.
pub struct Proj<M, F> {
    pub(crate) mutator: M,
    pub(crate) f: F,
}

impl<M, F, T, U> Mutate<T> for Proj<M, F>
where
    M: Mutate<U>,
    F: FnMut(&mut T) -> &mut U,
{
    #[inline]
    fn mutate(&mut self, muts: &mut MutationSet, value: &mut T) -> Result<()> {
        self.mutator.mutate(muts, (self.f)(value))
    }
}

/// A mutator that always produces the same, given value.
///
/// This is useful for providing base cases that feed into other mutator
/// combinators, like [`or`][Mutate::or].
///
/// See the [`just`] function for more information.
#[derive(Clone, Debug, Default)]
pub struct Just<T> {
    pub(crate) value: T,
    exhausted: bool,
}

/// Create a mutator that always produces the same, given value.
///
/// This is useful for providing base cases that feed into other mutator
/// combinators, like [`or`][Mutate::or].
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, MutationBuilder};
///
/// let mut mutator = m::just(42).or(m::range(1..=10));
///
/// let mut x = 0;
///
/// let mut mtn = MutationBuilder::new();
/// for _ in 0..5 {
///     mtn.mutate_with(&mut mutator, &mut x)?;
///     println!("mutated x is {x}");
/// }
///
/// // Example output:
/// //
/// //     mutated x is 9
/// //     mutated x is 42
/// //     mutated x is 4
/// //     mutated x is 6
/// //     mutated x is 4
/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
pub fn just<T>(value: T) -> Just<T> {
    Just {
        value,
        exhausted: false,
    }
}

impl<T> Mutate<T> for Just<T>
where
    T: Clone,
{
    #[inline]
    fn mutate(&mut self, muts: &mut MutationSet<'_>, value: &mut T) -> Result<()> {
        if !self.exhausted {
            muts.mutation(|_| {
                *value = self.value.clone();
                self.exhausted = true;
                Ok(())
            })?;
        }
        Ok(())
    }
}

impl<T> Generate<T> for Just<T>
where
    T: Clone,
{
    fn generate(&mut self, _ctx: &mut MutationContext) -> Result<T> {
        Ok(self.value.clone())
    }
}
