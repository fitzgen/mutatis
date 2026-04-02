use super::*;

/// A mutator for `core::cmp::Reverse<T>` values.
///
/// See the [`reverse()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct Reverse<M> {
    mutator: M,
}

/// Create a new mutator for `core::cmp::Reverse<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use core::cmp::Reverse;
///
/// let mut mutator = m::reverse(m::u32());
/// let mut session = Session::new();
///
/// let mut value = Reverse(42u32);
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = Reverse(2376156147)
/// //     value = Reverse(3048386628)
/// //     value = Reverse(947280344)
/// //     value = Reverse(3553606103)
/// //     value = Reverse(3137940673)
/// # Ok(())
/// # }
/// ```
pub fn reverse<M>(mutator: M) -> Reverse<M> {
    Reverse { mutator }
}

impl<M, T> Mutate<core::cmp::Reverse<T>> for Reverse<M>
where
    M: Mutate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::cmp::Reverse<T>) -> Result<()> {
        self.mutator.mutate(c, &mut value.0)
    }
}

impl<M, T> Generate<core::cmp::Reverse<T>> for Reverse<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::cmp::Reverse<T>> {
        Ok(core::cmp::Reverse(self.mutator.generate(ctx)?))
    }
}

impl<T> DefaultMutate for core::cmp::Reverse<T>
where
    T: DefaultMutate,
{
    type DefaultMutate = Reverse<T::DefaultMutate>;
}
