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
    fn mutation_count(
        &self,
        value: &core::result::Result<T, E>,
        shrink: bool,
    ) -> core::option::Option<u32> {
        match value {
            Ok(x) => {
                let mut count = 0u32;
                // Mutate inner Ok value.
                count += self.ok_mutator.mutation_count(x, shrink)?;
                // Generate an Err value.
                count += !shrink as u32;
                Some(count)
            }
            Err(e) => {
                let mut count = 0u32;
                // Mutate inner Err value.
                count += self.err_mutator.mutation_count(e, shrink)?;
                // Generate an Ok value.
                count += 1;
                Some(count)
            }
        }
    }

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

impl<M, N, T, E> Generate<core::result::Result<T, E>> for Result<M, N>
where
    M: Generate<T>,
    N: Generate<E>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> crate::Result<core::result::Result<T, E>> {
        if ctx.rng().gen_bool() {
            Ok(Ok(self.ok_mutator.generate(ctx)?))
        } else {
            Ok(Err(self.err_mutator.generate(ctx)?))
        }
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
