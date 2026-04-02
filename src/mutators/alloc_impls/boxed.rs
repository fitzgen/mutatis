use super::*;

/// The default mutator for `Box<T>` values.
///
/// See the [`boxed()`] function to create new instances and for example usage.
#[derive(Clone, Debug, Default)]
pub struct Boxed<M> {
    mutator: M,
}

/// Create a new mutator for `Box<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut value: Box<u32> = Box::new(0);
///
/// let mut mutator = m::boxed(m::mrange(100..=199));
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value}");
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
pub fn boxed<M>(mutator: M) -> Boxed<M> {
    Boxed { mutator }
}

impl<M, T> Mutate<alloc::boxed::Box<T>> for Boxed<M>
where
    M: Mutate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut alloc::boxed::Box<T>) -> Result<()> {
        self.mutator.mutate(c, value.as_mut())
    }
}

impl<M, T> Generate<alloc::boxed::Box<T>> for Boxed<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<alloc::boxed::Box<T>> {
        Ok(alloc::boxed::Box::new(self.mutator.generate(ctx)?))
    }
}

impl<T> DefaultMutate for alloc::boxed::Box<T>
where
    T: DefaultMutate,
    T::DefaultMutate: Generate<T>,
{
    type DefaultMutate = Boxed<T::DefaultMutate>;
}
