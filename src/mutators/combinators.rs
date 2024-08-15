use super::*;
use crate::Result;

/// A mutator combinator for applying one of many mutators.
///
/// This combinator is useful when you have a set of mutators and you want to
/// randomly pick one to apply.
///
/// Create instances of this combinator using the [`one_of`] function.
pub struct OneOf<Ms> {
    inner: Ms,
}

mod one_of_private {
    use super::*;

    pub trait TupleOfMutatorsImpl<T> {
        fn len(&self) -> usize;

        fn with_entry<F>(&mut self, i: usize, f: F) -> Result<()>
        where
            F: WithEntry<T>;
    }

    pub trait WithEntry<T> {
        fn with<M>(&mut self, mutator: &mut M) -> Result<()>
        where
            M: Mutator<T>;
    }

    macro_rules! impl_tuple_of_mutators {
        (
            $(
                (
                    $( $ty_param:ident ),*
                ) ;
            )*
        ) => {
            $(
                impl< T, $( $ty_param ),* > TupleOfMutatorsImpl<T> for ( $( $ty_param , )* )
                where
                    $( $ty_param: Mutator<T>, )*
                {
                    fn len(&self) -> usize {
                        impl_tuple_of_mutators!(@len $( $ty_param )*)
                    }

                    fn with_entry<F>(&mut self, i: usize, mut f: F) -> Result<()>
                    where
                        F: WithEntry<T>
                    {
                        let mut j = 0;

                        #[allow(non_snake_case)]
                        let ( $( ref mut $ty_param , )* ) = *self;

                        impl_tuple_of_mutators!(@with_entry i, j, f, $( $ty_param )*)
                    }
                }
            )*
        };

        ( @len ) => { 0 };
        ( @len $head:ident $( $tail:ident )* ) => { 1 + impl_tuple_of_mutators!(@len $( $tail )*) };

        ( @with_entry $i:ident , $j:ident , $f:ident , ) => {{
            let _ = ($i, &mut $j, &mut $f);
            unreachable!()
        }};
        ( @with_entry $i:ident , $j:ident , $f:ident , $head:ident $( $tail:ident )* ) => {{
            if $i == $j {
                return $f.with($head);
            }
            $j += 1;
            impl_tuple_of_mutators!(@with_entry $i, $j, $f, $( $tail )* )
        }};
    }

    impl_tuple_of_mutators! {
        ();
        (M0);
        (M0, M1);
        (M0, M1, M2);
        (M0, M1, M2, M3);
        (M0, M1, M2, M3, M4);
        (M0, M1, M2, M3, M4, M5);
        (M0, M1, M2, M3, M4, M5, M6);
        (M0, M1, M2, M3, M4, M5, M6, M7);
        (M0, M1, M2, M3, M4, M5, M6, M7, M8);
        (M0, M1, M2, M3, M4, M5, M6, M7, M8, M9);
        (M0, M1, M2, M3, M4, M5, M6, M7, M8, M9, M10);
        (M0, M1, M2, M3, M4, M5, M6, M7, M8, M9, M10, M11);
        (M0, M1, M2, M3, M4, M5, M6, M7, M8, M9, M10, M11, M12);
        (M0, M1, M2, M3, M4, M5, M6, M7, M8, M9, M10, M11, M12, M13);
        (M0, M1, M2, M3, M4, M5, M6, M7, M8, M9, M10, M11, M12, M13, M14);
        (M0, M1, M2, M3, M4, M5, M6, M7, M8, M9, M10, M11, M12, M13, M14, M15);
    }
}

/// A trait for tuples of mutators.
///
/// This trait is implemented for tuples of up to 16 mutators.
pub trait TupleOfMutators<T>: one_of_private::TupleOfMutatorsImpl<T> {}
impl<T, U> TupleOfMutators<T> for U where U: one_of_private::TupleOfMutatorsImpl<T> {}

/// Create a mutator that applies one of many sub-mutators.
///
/// This is useful when you have N mutations you could apply, and want to
/// randomly choose only one of them to actually apply.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, MutationContext, Mutator};
///
/// let mut mutator = m::one_of((
///     m::range(1..=10),
///     m::range(100..=200),
/// ));
///
/// let mut x = u32::MAX;
///
/// let mut context = MutationContext::default();
/// for _ in 0..5 {
///     mutator.mutate(&mut context, &mut x)?;
///     println!("mutated x is {x}");
/// }
///
/// // Example output:
/// //
/// //     mutated x is 4
/// //     mutated x is 104
/// //     mutated x is 145
/// //     mutated x is 2
/// //     mutated x is 102
/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
#[inline]
#[must_use = "mutators do nothing unless you invoke their `mutate` method"]
pub fn one_of<Ms>(tuple_of_mutators: Ms) -> OneOf<Ms> {
    OneOf {
        inner: tuple_of_mutators,
    }
}

impl<Ms, T> Mutator<T> for OneOf<Ms>
where
    Ms: TupleOfMutators<T>,
{
    fn mutate(&mut self, context: &mut MutationContext, value: &mut T) -> Result<()> {
        struct MutateEntry<'a, 'b, T> {
            context: &'a mut MutationContext,
            value: &'b mut T,
        }

        impl<T> one_of_private::WithEntry<T> for MutateEntry<'_, '_, T> {
            fn with<M>(&mut self, mutator: &mut M) -> Result<()>
            where
                M: Mutator<T>,
            {
                mutator.mutate(self.context, self.value)
            }
        }

        let len = self.inner.len();
        for _ in 0..len {
            let i = context.rng().gen_index(len).unwrap();
            match self.inner.with_entry(i, MutateEntry { context, value }) {
                Err(e) if !e.is_mutator_exhausted() => {
                    // Try again; hopefully we'll pick a different mutator that
                    // isn't exhausted.
                    continue;
                }
                result => return result,
            }
        }

        Err(Error::mutator_exhausted())
    }
}

// TODO: implement GenerativeMutator for OneOf

// TODO: implement FusedMutator for OneOf

/// A mutator combinator for mapping a function over the mutations produced by
/// another mutator.
///
/// See the [`map_mutate`][Mutator::map_mutate] method on the [`Mutator`] trait
/// for details and example usage.
#[derive(Clone, Debug)]
pub struct MapMutate<M, F> {
    pub(crate) mutator: M,
    pub(crate) f: F,
}

impl<M, F, T> Mutator<T> for MapMutate<M, F>
where
    M: Mutator<T>,
    F: FnMut(&mut MutationContext, &mut T) -> Result<()>,
{
    fn mutate(&mut self, context: &mut MutationContext, value: &mut T) -> Result<()> {
        self.mutator.mutate(context, value)?;
        (self.f)(context, value)
    }
}

impl<M, F, T> FusedMutator<T> for MapMutate<M, F>
where
    M: FusedMutator<T>,
    F: FnMut(&mut MutationContext, &mut T) -> Result<()>,
{
}

/// A mutator combinator for filtering the mutations produced by another
/// mutator.
///
/// See the [`filter_mutate`][Mutator::filter_mutate] method on the [`Mutator`]
/// trait for more information.
#[derive(Clone, Debug)]
pub struct FilterMutate<M, F> {
    pub(crate) mutator: M,
    pub(crate) f: F,
}

impl<M, F, T> Mutator<T> for FilterMutate<M, F>
where
    M: Mutator<T>,
    F: FnMut(&MutationContext, &T) -> bool,
{
    fn mutate(&mut self, context: &mut MutationContext, value: &mut T) -> Result<()> {
        loop {
            self.mutator.mutate(context, value)?;
            if (self.f)(context, value) {
                return Ok(());
            }
        }
    }
}

/// A mutator combinator for applying another mutator a fixed number of times in
/// a "single" mutation.
///
/// See the [`mutate_n`][Mutator::mutate_n] method on the [`Mutator`] trait for
/// more information.
#[derive(Clone, Debug)]
pub struct MutateN<M> {
    pub(crate) mutator: M,
    pub(crate) n: u32,
}

impl<M, T> Mutator<T> for MutateN<M>
where
    M: Mutator<T>,
{
    fn mutate(&mut self, context: &mut MutationContext, value: &mut T) -> Result<()> {
        let mut did_any = false;
        for _ in 0..self.n {
            match self.mutator.mutate(context, value) {
                Ok(()) => did_any = true,
                Err(e) if e.is_mutator_exhausted() && did_any => return Ok(()),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

/// A mutator combinator for projecting a value to a sub-value and applying a
/// mutator to that sub-value.
///
/// See the [`proj`][Mutator::proj] method on the [`Mutator`] trait to construct
/// this type, for examples, for more information.
pub struct Proj<M, F> {
    pub(crate) mutator: M,
    pub(crate) f: F,
}

impl<M, F, T, U> Mutator<T> for Proj<M, F>
where
    M: Mutator<U>,
    F: FnMut(&mut T) -> &mut U,
{
    #[inline]
    fn mutate(&mut self, context: &mut MutationContext, value: &mut T) -> Result<()> {
        self.mutator.mutate(context, (self.f)(value))
    }
}

/// A mutator implemented by a function that mutates a value.
///
/// This is useful when you have a function that mutates a value and you want to
/// use it as a mutator.
///
/// See the [`from_fn`] function for more information.
#[derive(Clone, Debug, Default)]
pub struct FromFn<F> {
    pub(crate) f: F,
}

/// Create a mutator from a function that mutates a value.
///
/// This is useful when you have a function that mutates a value and you want to
/// use it as a mutator.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, MutationContext, Mutator, Result};
///
/// let mut mutator = m::from_fn(|_context: &mut MutationContext, x: &mut u32| -> Result<()> {
///     *x = 42;
///     Ok(())
/// });
///
/// let mut x = 0;
///
/// let mut context = MutationContext::default();
/// mutator.mutate(&mut context, &mut x)?;
///
/// assert_eq!(x, 42);
/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
pub fn from_fn<F>(f: F) -> FromFn<F> {
    FromFn { f }
}

impl<F, T> Mutator<T> for FromFn<F>
where
    F: FnMut(&mut MutationContext, &mut T) -> Result<()>,
{
    #[inline]
    fn mutate(&mut self, context: &mut MutationContext, value: &mut T) -> Result<()> {
        (self.f)(context, value)
    }
}

/// A mutator that always produces the same, given value.
///
/// This is useful for providing base cases that feed into other mutator
/// combinators, like [`one_of`].
///
/// See the [`just`] function for more information.
#[derive(Clone, Debug, Default)]
pub struct Just<T> {
    pub(crate) value: T,
}

/// Create a mutator that always produces the same, given value.
///
/// This is useful for providing base cases that feed into other mutator
/// combinators, like [`one_of`].
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, MutationContext, Mutator};
///
/// let mut mutator = m::one_of((
///     m::just(42),
///     m::range(1..=10),
/// ));
///
/// let mut x = 0;
///
/// let mut context = MutationContext::default();
/// for _ in 0..50 {
///     mutator.mutate(&mut context, &mut x)?;
///     println!("mutated x is {x}");
/// }
///
/// // Example output:
/// //
/// //     mutated x is 9
/// //     mutated x is 42
/// //     mutated x is 42
/// //     mutated x is 4
/// //     mutated x is 42
/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
pub fn just<T>(value: T) -> Just<T> {
    Just { value }
}

impl<T> Mutator<T> for Just<T>
where
    T: Clone,
{
    #[inline]
    fn mutate(&mut self, _context: &mut MutationContext, value: &mut T) -> Result<()> {
        *value = self.value.clone();
        Ok(())
    }
}
