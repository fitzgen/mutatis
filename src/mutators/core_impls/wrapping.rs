use super::*;

/// A mutator for `core::num::Wrapping<T>` values.
///
/// See the [`wrapping()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct Wrapping<M> {
    mutator: M,
}

/// Create a new mutator for `core::num::Wrapping<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use core::num::Wrapping;
///
/// let mut mutator = m::wrapping(m::u32());
/// let mut session = Session::new();
///
/// let mut value = Wrapping(42u32);
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value}");
/// }
///
/// // Example output:
/// //
/// //     value = Wrapping(3231734609)
/// //     value = Wrapping(965025742)
/// //     value = Wrapping(3762627074)
/// //     value = Wrapping(1484783241)
/// //     value = Wrapping(2525163492)
/// # Ok(())
/// # }
/// ```
pub fn wrapping<M>(mutator: M) -> Wrapping<M> {
    Wrapping { mutator }
}

impl<M, T> Mutate<core::num::Wrapping<T>> for Wrapping<M>
where
    M: Mutate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::num::Wrapping<T>) -> Result<()> {
        self.mutator.mutate(c, &mut value.0)
    }
}

impl<M, T> Generate<core::num::Wrapping<T>> for Wrapping<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::num::Wrapping<T>> {
        Ok(core::num::Wrapping(self.mutator.generate(ctx)?))
    }
}

impl<T> DefaultMutate for core::num::Wrapping<T>
where
    T: DefaultMutate,
{
    type DefaultMutate = Wrapping<T::DefaultMutate>;
}
