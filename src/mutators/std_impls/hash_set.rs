use super::*;
use std::mem;

/// The default mutator for `HashSet<T>` values.
///
/// See the [`hash_set()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct HashSet<M> {
    mutator: M,
}

/// Create a new mutator for `HashSet<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::collections::HashSet;
///
/// let mut items: HashSet<Vec<u32>> = HashSet::new();
///
/// let mut mutator = m::hash_set(m::vec(m::mrange(100..=199)));
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut items)?;
///     println!("items = {items:?}");
/// }
///
/// // Example output:
/// //
/// //     items = {[146]}
/// //     items = {[194]}
/// //     items = {[164, 194]}
/// //     items = {[164, 194], [122]}
/// //     items = {[164, 194], [122, 118]}
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn hash_set<M>(mutator: M) -> HashSet<M> {
    HashSet { mutator }
}

impl<M, T> Mutate<std::collections::HashSet<T>> for HashSet<M>
where
    M: Generate<T>,
    T: Eq + Hash,
{
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut std::collections::HashSet<T>,
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
                let target = ctx.rng().gen_index(value.len()).unwrap();
                let mut i = 0;
                value.retain(|_| {
                    let keep = i != target;
                    i += 1;
                    keep
                });
                Ok(())
            })?;
        }

        // Mutate a random element.
        if !value.is_empty() {
            let elems = mem::take(value).into_iter();
            value.reserve(elems.len());

            let mut early_exit = None;
            for mut elem in elems {
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

impl<M, T> Generate<std::collections::HashSet<T>> for HashSet<M>
where
    M: Generate<T>,
    T: Eq + Hash,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<std::collections::HashSet<T>> {
        self.generate_via_mutate(ctx, 1)
    }
}

impl<T> DefaultMutate for std::collections::HashSet<T>
where
    T: DefaultMutate + Eq + Hash,
    T::DefaultMutate: Generate<T>,
{
    type DefaultMutate = HashSet<T::DefaultMutate>;
}
