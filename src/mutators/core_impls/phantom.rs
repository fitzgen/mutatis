use super::*;
use core::marker;

/// A mutator for `core::marker::PhantomData<T>` values.
///
/// See the [`phantom_data()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug)]
pub struct PhantomDataMutator<T> {
    _phantom: marker::PhantomData<T>,
}

impl<T> Default for PhantomDataMutator<T> {
    fn default() -> Self {
        PhantomDataMutator {
            _phantom: marker::PhantomData,
        }
    }
}

/// Create a new mutator for `core::marker::PhantomData<T>` values.
///
/// # Example
///
/// ```
/// use mutatis::{mutators as m, Generate, Session};
/// use core::marker::PhantomData;
///
/// let mut mutator = m::phantom_data::<u32>();
/// let mut session = Session::new();
///
/// let value: PhantomData<u32> = session.generate_with(&mut mutator).unwrap();
///
/// assert_eq!(value, PhantomData);
/// ```
pub fn phantom_data<T>() -> PhantomDataMutator<T> {
    PhantomDataMutator {
        _phantom: marker::PhantomData,
    }
}

impl<T> Mutate<marker::PhantomData<T>> for PhantomDataMutator<T> {
    #[inline]
    fn mutate(&mut self, _c: &mut Candidates, _value: &mut marker::PhantomData<T>) -> Result<()> {
        Ok(())
    }
}

impl<T> Generate<marker::PhantomData<T>> for PhantomDataMutator<T> {
    #[inline]
    fn generate(&mut self, _ctx: &mut Context) -> Result<marker::PhantomData<T>> {
        Ok(marker::PhantomData)
    }
}

impl<T> DefaultMutate for marker::PhantomData<T> {
    type DefaultMutate = PhantomDataMutator<T>;
}
