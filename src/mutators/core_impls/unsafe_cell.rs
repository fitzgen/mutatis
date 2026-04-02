use super::*;

/// A mutator for `core::cell::UnsafeCell<T>` values.
///
/// See the [`unsafe_cell()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct UnsafeCellMutator<M> {
    mutator: M,
}

/// Create a new mutator for `core::cell::UnsafeCell<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use core::cell::UnsafeCell;
///
/// let mut mutator = m::unsafe_cell(m::u32());
/// let mut session = Session::new();
///
/// let mut value = UnsafeCell::new(42u32);
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {:?}", value.get_mut());
/// }
///
/// // Example output:
/// //
/// //     value = 4107414004
/// //     value = 683392052
/// //     value = 479040106
/// //     value = 198197041
/// //     value = 3443243697
/// # Ok(())
/// # }
/// ```
pub fn unsafe_cell<M>(mutator: M) -> UnsafeCellMutator<M> {
    UnsafeCellMutator { mutator }
}

impl<M, T> Mutate<core::cell::UnsafeCell<T>> for UnsafeCellMutator<M>
where
    M: Mutate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::cell::UnsafeCell<T>) -> Result<()> {
        self.mutator.mutate(c, value.get_mut())
    }
}

impl<M, T> Generate<core::cell::UnsafeCell<T>> for UnsafeCellMutator<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::cell::UnsafeCell<T>> {
        Ok(core::cell::UnsafeCell::new(self.mutator.generate(ctx)?))
    }
}

impl<T> DefaultMutate for core::cell::UnsafeCell<T>
where
    T: DefaultMutate,
{
    type DefaultMutate = UnsafeCellMutator<T::DefaultMutate>;
}
