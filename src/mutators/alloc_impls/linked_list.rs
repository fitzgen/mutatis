use super::*;

/// The default mutator for `LinkedList<T>` values.
///
/// See the [`linked_list()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct LinkedList<M> {
    mutator: M,
}

/// Create a new mutator for `LinkedList<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::collections::LinkedList;
///
/// let mut items: LinkedList<u32> = LinkedList::new();
///
/// let mut mutator = m::linked_list(m::mrange(100..=199));
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
/// //     items = []
/// //     items = [124]
/// //     items = [124, 122]
/// //     items = [122]
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn linked_list<M>(mutator: M) -> LinkedList<M> {
    LinkedList { mutator }
}

impl<M, T> Mutate<alloc::collections::LinkedList<T>> for LinkedList<M>
where
    M: Generate<T> + Mutate<T>,
{
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut alloc::collections::LinkedList<T>,
    ) -> Result<()> {
        // Add an element.
        if !c.shrink() {
            c.mutation(|ctx| {
                let elem = self.mutator.generate(ctx)?;
                if ctx.rng().gen_bool() {
                    value.push_front(elem);
                } else {
                    value.push_back(elem);
                }
                Ok(())
            })?;
        }

        // Remove an element.
        if !value.is_empty() {
            c.mutation(|ctx| {
                if ctx.rng().gen_bool() {
                    value.pop_front();
                } else {
                    value.pop_back();
                }
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

impl<M, T> Generate<alloc::collections::LinkedList<T>> for LinkedList<M>
where
    M: Generate<T> + Mutate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<alloc::collections::LinkedList<T>> {
        self.generate_via_mutate(ctx, 1)
    }
}

impl<T> DefaultMutate for alloc::collections::LinkedList<T>
where
    T: DefaultMutate,
    T::DefaultMutate: Generate<T>,
{
    type DefaultMutate = LinkedList<T::DefaultMutate>;
}
