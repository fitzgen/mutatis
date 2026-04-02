use super::*;

/// The default mutator for `BinaryHeap<T>` values.
///
/// See the [`binary_heap()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct BinaryHeap<M> {
    mutator: M,
}

/// Create a new mutator for `BinaryHeap<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::collections::BinaryHeap;
///
/// let mut items: BinaryHeap<u32> = BinaryHeap::new();
///
/// let mut mutator = m::binary_heap(m::mrange(100..=199));
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut items)?;
///     println!("items = {:?}", items.iter().collect::<Vec<_>>());
/// }
///
/// // Example output:
/// //
/// //     items = [146]
/// //     items = [145]
/// //     items = []
/// //     items = [124]
/// //     items = [124, 122]
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn binary_heap<M>(mutator: M) -> BinaryHeap<M> {
    BinaryHeap { mutator }
}

impl<M, T> Mutate<alloc::collections::BinaryHeap<T>> for BinaryHeap<M>
where
    M: Generate<T>,
    T: Ord,
{
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut alloc::collections::BinaryHeap<T>,
    ) -> Result<()> {
        // Add an element.
        if !c.shrink() {
            c.mutation(|ctx| {
                let elem = self.mutator.generate(ctx)?;
                value.push(elem);
                Ok(())
            })?;
        }

        // Remove an element.
        if !value.is_empty() {
            c.mutation(|_ctx| {
                value.pop();
                Ok(())
            })?;
        }

        // Replace a random element.
        if !value.is_empty() {
            c.mutation(|ctx| {
                let mut vec = core::mem::take(value).into_vec();
                let index = ctx.rng().gen_index(vec.len()).unwrap();
                vec[index] = self.mutator.generate(ctx)?;
                *value = alloc::collections::BinaryHeap::from(vec);
                Ok(())
            })?;
        }

        Ok(())
    }
}

impl<M, T> Generate<alloc::collections::BinaryHeap<T>> for BinaryHeap<M>
where
    M: Generate<T>,
    T: Ord,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<alloc::collections::BinaryHeap<T>> {
        self.generate_via_mutate(ctx, 1)
    }
}

impl<T> DefaultMutate for alloc::collections::BinaryHeap<T>
where
    T: DefaultMutate + Ord,
    T::DefaultMutate: Generate<T>,
{
    type DefaultMutate = BinaryHeap<T::DefaultMutate>;
}
