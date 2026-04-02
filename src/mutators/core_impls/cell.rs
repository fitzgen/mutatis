use super::*;

/// The default mutator for `Cell<T>` values.
///
/// See the [`cell()`] function to create new instances and for example usage.
#[derive(Clone, Debug, Default)]
pub struct Cell<M> {
    mutator: M,
}

/// Create a new mutator for `Cell<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::cell::Cell;
///
/// let mut value = Cell::new(0u32);
///
/// let mut mutator = m::cell(m::mrange(100..=199));
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {:?}", value.get());
/// }
///
/// // Example output:
/// //
/// //     value = 144
/// //     value = 146
/// //     value = 189
/// //     value = 194
/// //     value = 145
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn cell<M>(mutator: M) -> Cell<M> {
    Cell { mutator }
}

impl<M, T> Mutate<core::cell::Cell<T>> for Cell<M>
where
    M: Mutate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::cell::Cell<T>) -> Result<()> {
        self.mutator.mutate(c, value.get_mut())
    }
}

impl<M, T> Generate<core::cell::Cell<T>> for Cell<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::cell::Cell<T>> {
        Ok(core::cell::Cell::new(self.mutator.generate(ctx)?))
    }
}

impl<T> DefaultMutate for core::cell::Cell<T>
where
    T: DefaultMutate,
{
    type DefaultMutate = Cell<T::DefaultMutate>;
}
