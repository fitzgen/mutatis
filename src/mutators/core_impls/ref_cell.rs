use super::*;

/// The default mutator for `RefCell<T>` values.
///
/// See the [`ref_cell()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct RefCell<M> {
    mutator: M,
}

/// Create a new mutator for `RefCell<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::cell::RefCell;
///
/// let mut value = RefCell::new(0u32);
///
/// let mut mutator = m::ref_cell(m::mrange(100..=199));
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {:?}", value.borrow());
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
pub fn ref_cell<M>(mutator: M) -> RefCell<M> {
    RefCell { mutator }
}

impl<M, T> Mutate<core::cell::RefCell<T>> for RefCell<M>
where
    M: Mutate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::cell::RefCell<T>) -> Result<()> {
        self.mutator.mutate(c, value.get_mut())
    }
}

impl<M, T> Generate<core::cell::RefCell<T>> for RefCell<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::cell::RefCell<T>> {
        Ok(core::cell::RefCell::new(self.mutator.generate(ctx)?))
    }
}

impl<T> DefaultMutate for core::cell::RefCell<T>
where
    T: DefaultMutate,
{
    type DefaultMutate = RefCell<T::DefaultMutate>;
}
