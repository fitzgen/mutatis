use super::*;

/// A mutator for `Option<T>` values.
///
/// See the [`option()`] function to create a new `Option` mutator and for
/// example usage.
#[derive(Clone, Debug, Default)]
pub struct Option<M> {
    mutator: M,
}

/// Create a new mutator for `Option<T>` values.
///
/// # Example
///
/// ```
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::option(m::u32());
/// let mut session = Session::new();
///
/// let mut value = Some(36);
/// session.mutate_with(&mut mutator, &mut value).unwrap();
///
/// println!("mutated option is {value:?}");
/// ```
pub fn option<M>(mutator: M) -> Option<M> {
    Option { mutator }
}

impl<M, T> Mutate<core::option::Option<T>> for Option<M>
where
    M: Generate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::option::Option<T>) -> Result<()> {
        if c.shrink() && value.is_none() {
            return Ok(());
        }

        match value.as_mut() {
            None => c.mutation(|ctx| Ok(*value = Some(self.mutator.generate(ctx)?))),
            Some(v) => {
                self.mutator.mutate(c, v)?;
                c.mutation(|_| Ok(*value = None))
            }
        }
    }
}

impl<T> DefaultMutate for core::option::Option<T>
where
    T: DefaultMutate,
    T::DefaultMutate: Generate<T>,
{
    type DefaultMutate = Option<T::DefaultMutate>;
}

/// A mutator for `Option<T>` values that always produces `Some` values.
///
/// See the [`some()`] function to create a new `Some` mutator and for example
/// usage.
pub struct Some<M> {
    mutator: M,
}

/// Create a new mutator for `Option<T>` values that always produces `Some`
/// values.
///
/// # Example
///
/// ```
/// # fn foo() -> Result<(), mutatis::Error> {
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::some(m::u32());
/// let mut session = Session::new();
///
/// let mut value = None;
/// for _ in 0..10 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     assert!(value.is_some());
/// }
/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
pub fn some<M>(mutator: M) -> Some<M> {
    Some { mutator }
}

impl<M, T> Mutate<core::option::Option<T>> for Some<M>
where
    M: Generate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::option::Option<T>) -> Result<()> {
        match value.as_mut() {
            None if c.shrink() => Ok(()),
            None => c.mutation(|ctx| Ok(*value = Some(self.mutator.generate(ctx)?))),
            Some(v) => self.mutator.mutate(c, v),
        }
    }
}

/// A mutator for `Option<T>` values that always produces `None` values.
///
/// See the [`none()`] function to create a new `None` mutator and for example
/// usage.
pub struct None {
    _private: (),
}

/// Create a new mutator for `Option<T>` values that always produces `None`
/// values.
///
/// # Example
///
/// ```
/// # fn foo() -> Result<(), mutatis::Error> {
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::none();
/// let mut session = Session::new();
///
/// let mut value = Some(36);
///
/// session.mutate_with(&mut mutator, &mut value)?;
/// assert!(value.is_none());
/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
pub fn none() -> None {
    None { _private: () }
}

impl<T> Mutate<core::option::Option<T>> for None {
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::option::Option<T>) -> Result<()> {
        if value.is_some() {
            c.mutation(|_| Ok(*value = None))?;
        }
        Ok(())
    }
}
