use super::*;
use core::mem;

/// The default mutator for `BTreeSet<T>` values.
///
/// See the [`btree_set()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct BTreeSet<M> {
    mutator: M,
}

/// Create a new mutator for `BTreeSet<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::collections::BTreeSet;
///
/// let mut items: BTreeSet<u32> = BTreeSet::new();
///
/// let mut mutator = m::btree_set(m::mrange(100..=199));
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut items)?;
///     println!("items = {items:?}");
/// }
///
/// // Example output:
/// //
/// //     items = {146}
/// //     items = {145}
/// //     items = {}
/// //     items = {197}
/// //     items = {122, 197}
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn btree_set<M>(mutator: M) -> BTreeSet<M> {
    BTreeSet { mutator }
}

impl<M, T> Mutate<alloc::collections::BTreeSet<T>> for BTreeSet<M>
where
    M: Generate<T>,
    T: Ord,
{
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut alloc::collections::BTreeSet<T>,
    ) -> Result<()> {
        // Add an element.
        if !c.shrink() {
            c.mutation(|ctx| {
                let elem = self.mutator.generate(ctx)?;
                value.insert(elem);
                Ok(())
            })?;
        }

        // Remove an element.
        if !value.is_empty() {
            c.mutation(|ctx| {
                if ctx.rng().gen_bool() {
                    value.pop_first();
                } else {
                    value.pop_last();
                }
                Ok(())
            })?;
        }

        // Mutate a random element.
        if !value.is_empty() {
            let mut early_exit = None;

            for mut elem in mem::take(value) {
                if early_exit.is_none() {
                    match self.mutator.mutate(c, &mut elem) {
                        Ok(()) => {}
                        Err(e) if e.is_early_exit() => {
                            early_exit = Some(e);
                        }
                        Err(e) => return Err(e),
                    }
                }

                value.insert(elem);
            }

            if let Some(e) = early_exit {
                return Err(e);
            }
        }

        Ok(())
    }
}

impl<M, T> Generate<alloc::collections::BTreeSet<T>> for BTreeSet<M>
where
    M: Generate<T>,
    T: Ord,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<alloc::collections::BTreeSet<T>> {
        self.generate_via_mutate(ctx, 1)
    }
}

impl<T> DefaultMutate for alloc::collections::BTreeSet<T>
where
    T: DefaultMutate + Ord,
    T::DefaultMutate: Generate<T>,
{
    type DefaultMutate = BTreeSet<T::DefaultMutate>;
}
