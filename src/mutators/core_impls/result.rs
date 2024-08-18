use super::*;

/// A mutator for `Result<T, E>`.
///
/// See the [`result`] function for creating new `Result` mutators and for
/// example usage.
#[derive(Clone, Debug, Default)]
pub struct Result<M, N> {
    ok_mutator: M,
    err_mutator: N,
}

/// Create a new `Result` mutator.
///
/// # Example
///
/// ```
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::result(m::u32(), m::i8());
/// let mut session = Session::new();
///
/// let mut value = Ok(1312);
/// session.mutate_with(&mut mutator, &mut value).unwrap();
///
/// println!("mutated result is {value:?}");
/// ```
pub fn result<M, N>(ok_mutator: M, err_mutator: N) -> Result<M, N> {
    Result {
        ok_mutator,
        err_mutator,
    }
}

impl<M, N, T, E> Mutate<core::result::Result<T, E>> for Result<M, N>
where
    M: Generate<T>,
    N: Generate<E>,
{
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut core::result::Result<T, E>,
    ) -> crate::Result<()> {
        match value {
            Ok(x) => {
                self.ok_mutator.mutate(c, x)?;
                if !c.shrink() {
                    c.mutation(|ctx| Ok(*value = Err(self.err_mutator.generate(ctx)?)))?;
                }
            }
            Err(e) => {
                self.err_mutator.mutate(c, e)?;
                c.mutation(|ctx| Ok(*value = Ok(self.ok_mutator.generate(ctx)?)))?;
            }
        }
        Ok(())
    }
}

impl<T, E> DefaultMutate for core::result::Result<T, E>
where
    T: DefaultMutate,
    T::DefaultMutate: Generate<T>,
    E: DefaultMutate,
    E::DefaultMutate: Generate<E>,
{
    type DefaultMutate = Result<T::DefaultMutate, E::DefaultMutate>;
}
