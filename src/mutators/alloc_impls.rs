//! `Mutate` implementations for `alloc` types.

use super::*;

/// The default mutator for `Vec<T>` values.
///
/// See the [`vec()`] function to create new instances and for example usage.
#[derive(Clone, Debug, Default)]
pub struct Vec<M> {
    mutator: M,
}

/// Create a new mutator for `Vec<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut items: Vec<u32> = vec![];
///
/// let mut mutator = m::vec(m::range(100..=199));
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut items)?;
///     println!("items = {items:?}");
/// }
///
/// // Example output:
/// //
/// //     items = [168]
/// //     items = []
/// //     items = [142]
/// //     items = [110]
/// //     items = [114, 110]
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn vec<M>(mutator: M) -> Vec<M> {
    Vec { mutator }
}

impl<M, T> Mutate<alloc::vec::Vec<T>> for Vec<M>
where
    M: Generate<T> + Mutate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut alloc::vec::Vec<T>) -> Result<()> {
        // Add an element.
        if !c.shrink() {
            c.mutation(|ctx| {
                let index = ctx.rng().gen_index(value.len() + 1).unwrap();
                let elem = self.mutator.generate(ctx)?;
                value.insert(index, elem);
                Ok(())
            })?;
        }

        // Remove an element.
        if !value.is_empty() {
            c.mutation(|ctx| {
                let index = ctx.rng().gen_index(value.len()).unwrap();
                value.remove(index);
                Ok(())
            })?;
        }

        // Mutate an existing element.
        for x in value {
            self.mutator.mutate(c, x)?;
        }

        Ok(())
    }
}

impl<T> DefaultMutate for alloc::vec::Vec<T>
where
    T: DefaultMutate,
    T::DefaultMutate: Generate<T>,
{
    type DefaultMutate = Vec<T::DefaultMutate>;
}
