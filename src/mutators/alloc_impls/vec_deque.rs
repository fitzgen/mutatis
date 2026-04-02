use super::*;

/// The default mutator for `VecDeque<T>` values.
///
/// See the [`vec_deque()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct VecDeque<M> {
    mutator: M,
}

/// Create a new mutator for `VecDeque<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::collections::VecDeque;
///
/// let mut items: VecDeque<u32> = VecDeque::new();
///
/// let mut mutator = m::vec_deque(m::mrange(100..=199));
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut items)?;
///     println!("items = {items:?}");
/// }
///
/// // Example output:
/// //
/// //     items = [146]
/// //     items = [194]
/// //     items = []
/// //     items = [124]
/// //     items = [129, 124]
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn vec_deque<M>(mutator: M) -> VecDeque<M> {
    VecDeque { mutator }
}

impl<M, T> Mutate<alloc::collections::VecDeque<T>> for VecDeque<M>
where
    M: Generate<T> + Mutate<T>,
{
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut alloc::collections::VecDeque<T>,
    ) -> Result<()> {
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

        // Swap two elements.
        if value.len() >= 2 {
            c.mutation(|ctx| {
                let i = ctx.rng().gen_index(value.len()).unwrap();
                let j = ctx.rng().gen_index(value.len()).unwrap();
                value.swap(i, j);
                Ok(())
            })?;
        }

        // Mutate an existing element.
        for x in value.iter_mut() {
            self.mutator.mutate(c, x)?;
        }

        Ok(())
    }
}

impl<M, T> Generate<alloc::collections::VecDeque<T>> for VecDeque<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<alloc::collections::VecDeque<T>> {
        self.generate_via_mutate(ctx, 1)
    }
}

impl<T> DefaultMutate for alloc::collections::VecDeque<T>
where
    T: DefaultMutate,
    T::DefaultMutate: Generate<T>,
{
    type DefaultMutate = VecDeque<T::DefaultMutate>;
}
