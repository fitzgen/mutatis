use super::*;
use alloc::borrow::ToOwned;

/// A mutator for `Cow<'a, B>` values.
///
/// See the [`cow()`] function to create new instances and for example usage.
#[derive(Clone, Debug, Default)]
pub struct CowMutator<M> {
    mutator: M,
}

/// Create a new mutator for `Cow<'a, B>` values.
///
/// The inner mutator operates on `<B as ToOwned>::Owned`.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::borrow::Cow;
///
/// let mut value: Cow<'_, str> = Cow::Borrowed("moo");
///
/// let mut mutator = m::cow(m::string(m::ascii_char()));
///
/// let mut session = Session::new().seed(36);
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = "moo9|s>\u{e}"
/// //     value = "mo"
/// //     value = ""
/// //     value = "\r"
/// //     value = "\rX"
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn cow<M>(mutator: M) -> CowMutator<M> {
    CowMutator { mutator }
}

impl<'a, M, B> Mutate<alloc::borrow::Cow<'a, B>> for CowMutator<M>
where
    B: ToOwned + ?Sized,
    M: Mutate<<B as ToOwned>::Owned>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut alloc::borrow::Cow<'a, B>) -> Result<()> {
        self.mutator.mutate(c, value.to_mut())
    }
}

impl<'a, M, B> Generate<alloc::borrow::Cow<'a, B>> for CowMutator<M>
where
    B: ToOwned + ?Sized,
    M: Generate<<B as ToOwned>::Owned>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<alloc::borrow::Cow<'a, B>> {
        Ok(alloc::borrow::Cow::Owned(self.mutator.generate(ctx)?))
    }
}

impl<'a, B> DefaultMutate for alloc::borrow::Cow<'a, B>
where
    B: ToOwned + ?Sized,
    <B as ToOwned>::Owned: DefaultMutate,
    <<B as ToOwned>::Owned as DefaultMutate>::DefaultMutate: Generate<<B as ToOwned>::Owned>,
{
    type DefaultMutate = CowMutator<<<B as ToOwned>::Owned as DefaultMutate>::DefaultMutate>;
}
