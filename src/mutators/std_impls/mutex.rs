use super::*;
use std::sync::Mutex;

/// The default mutator for `Mutex<T>` values.
///
/// See the [`mutex()`] function to create new instances and for example usage.
#[derive(Clone, Debug, Default)]
pub struct MutexMutator<M> {
    mutator: M,
}

/// Create a new mutator for `Mutex<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::sync::Mutex;
///
/// let mut value = Mutex::new(0u32);
///
/// let mut mutator = m::mutex(m::mrange(100..=199));
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {:?}", value.get_mut().unwrap());
/// }
///
/// // Example output:
/// //
/// //     value = 2214841768
/// //     value = 442456889
/// //     value = 395130196
/// //     value = 4037416745
/// //     value = 465646253
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn mutex<M>(mutator: M) -> MutexMutator<M> {
    MutexMutator { mutator }
}

impl<M, T> Mutate<Mutex<T>> for MutexMutator<M>
where
    M: Mutate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut Mutex<T>) -> Result<()> {
        self.mutator.mutate(c, value.get_mut().unwrap())
    }
}

impl<M, T> Generate<Mutex<T>> for MutexMutator<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<Mutex<T>> {
        Ok(Mutex::new(self.mutator.generate(ctx)?))
    }
}

impl<T> DefaultMutate for Mutex<T>
where
    T: DefaultMutate,
{
    type DefaultMutate = MutexMutator<T::DefaultMutate>;
}
