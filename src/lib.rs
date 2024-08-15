#![doc = include_str!("../README.md")]
#![no_std]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

mod error;
mod rng;

use core::ops;

pub use error::*;
pub use rng::Rng;

#[cfg(feature = "check")]
pub mod check;
pub mod mutators;

#[cfg(feature = "derive")]
pub use mutatis_derive::Mutator;

/// A builder for a `MutationContext`.
///
/// This type allows you to configure a `MutationContext`, doing things like
/// setting its RNG's seed, before building it.
///
/// Create a new builder via the [`MutationContext::builder()`] method.
///
/// # Example
///
/// ```
/// use mutatis::MutationContext;
///
/// let context = MutationContext::builder()
///     // Configure the RNG seed, changing which random mutation a mutator
///     // chooses to make.
///     .seed(0x12345678)
///     // Only perform mutations that "shrink" the test case.
///     .shrink(true)
///     // Finally, build the mutation context!
///     .build();
/// ```
#[derive(Clone, Debug, Default)]
pub struct MutationContextBuilder {
    context: MutationContext,
}

impl MutationContextBuilder {
    /// Set the seed for the random number generator.
    pub fn seed(mut self, seed: u64) -> Self {
        self.context.rng = Rng::new(seed);
        self
    }

    /// Set whether to only perform shrinking mutations or not.
    ///
    /// Defaults to `false`.
    pub fn shrink(mut self, shrink: bool) -> Self {
        self.context.shrink = shrink;
        self
    }

    /// Build the configured `MutationContext`.
    pub fn build(self) -> MutationContext {
        self.context
    }
}

/// The context for a set of mutations.
///
/// This context includes things like configuration options (whether to only
/// perform "shrinking" mutations or not) as well as a random number generator
/// (RNG) to help choose which mutation to perform when there are multiple
/// options (for example, choosing between adding, deleting, or modifying a
/// character in a string).
///
/// Every mutation operation is given a context.
///
/// Contexts may be reused across multiple mutations, if you want.
///
/// New contexts are created via getting a [`MutationContextBuilder`] from the
/// [`MutationContext::builder()`] method, configuring the builder, and then
/// calling its [`build()`][MutationContextBuilder::build] method to get the
/// newly-created context. See the documentation for [`MutationContextBuilder`]
/// for an example of building and configuring a new context.
#[derive(Clone, Debug, Default)]
pub struct MutationContext {
    rng: Rng,
    shrink: bool,
}

impl MutationContext {
    /// Create a new builder for a `MutationContext`.
    ///
    /// This allows you to configure mutations, such as setting the seed for the
    /// random number generator, before finally building the `MutationContext`.
    ///
    /// See the documentation for [`MutationContextBuilder`] for example usage.
    #[inline]
    #[must_use]
    pub fn builder() -> MutationContextBuilder {
        MutationContextBuilder::default()
    }

    /// Get this mutation context's random number generator.
    #[inline]
    #[must_use]
    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }

    /// Whether only shrinking mutations should be performed or not.
    ///
    /// When this method returns `true`, then mutator implementations should
    /// avoid performing any mutation which increases the size or complexity of
    /// the value that they are mutating.
    #[inline]
    #[must_use]
    pub fn shrink(&self) -> bool {
        self.shrink
    }
}

/// A trait for mutating values.
///
/// You can think of `Mutator<T>` as a streaming iterator of `T`s but instead of
/// internally containing and giving access to the `T`s, it takes them as an
/// argument and mutates them in place.
///
/// The main method is the `mutate` method, which takes a [`MutationContext`]
/// and a value, and then mutates the given value or returns an error.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, MutationContext, Mutator};
///
/// // Define a mutator.
/// let mut mutator = m::u32()
///     .map_mutate(|_context, x| {
///         *x = x.wrapping_mul(4);
///         Ok(())
///     })
///     .filter_mutate(|_context, x| !x.is_power_of_two());
///
/// // Mutate a value a bunch of times!
/// let mut x = 1234;
/// let mut context = MutationContext::default();
/// for _ in 0..5 {
///     mutator.mutate(&mut context, &mut x)?;
///     println!("mutated x is {x}");
/// }
///
/// // Example output:
/// //
/// //     mutated x is 2436583184
/// //     mutated x is 2032949584
/// //     mutated x is 2631247496
/// //     mutated x is 199875380
/// //     mutated x is 3751781284
/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
///
/// # Exhaustion
///
/// A mutator may become *exhausted*, meaning that it doesn't have any more
/// mutations it can perform for a given value. In this case, the mutator may
/// return an error of kind [`ErrorKind::MutatorExhausted`]. Many mutators are
/// effectively inexhaustible (or it would be prohibitively expensive to
/// precisely track whether they've emitted every possible mutation of a value,
/// like a mutator that flips a single bit in a `u32`) and therefore it is valid
/// for a mutator to never report exhaustion.
///
/// You may also ignore exhaustion errors via the
/// [`ResultExt::ignore_mutator_exhausted`] extension method.
///
/// # Many to Many
///
/// Note that the relationship between mutator types and mutated types is not
/// one-to-one: a single mutator type can mutate many different types, and a
/// single type can be mutated by many mutator types. This gives you the
/// flexibility to define new mutators for existing types (including those that
/// are not defined by your own crate).
///
/// ```
/// use mutatis::{mutators as m, MutationContext, Mutator, Result};
///
/// #[derive(Mutator)] // derive a default Mutator
/// pub struct Foo(u32);
///
/// // Define a second mutator for `Foo` by hand!
///
/// pub struct AlignedFooMutator{
///     inner: FooMutator,
///     alignment: u32,
/// }
///
/// impl Mutator<Foo> for AlignedFooMutator {
///     fn mutate(&mut self, context: &mut MutationContext, foo: &mut Foo) -> Result<()> {
///         self.inner.mutate(context, foo)?;
///
///         // Clear the bottom bits to keep the `Foo` "aligned".
///         debug_assert!(self.alignment.is_power_of_two());
///         let mask = !(self.alignment - 1);
///         foo.0 = foo.0 & mask;
///
///         Ok(())
///     }
/// }
/// ```
pub trait Mutator<T>
where
    T: ?Sized,
{
    // Required methods.

    /// Pseudorandomly mutate the given value.
    ///
    /// The `context` describes configured options for the mutation (such as,
    /// whether to only perform shrinking mutations) as well as a pseudorandom
    /// number generator.
    ///
    /// The `value` is modified in place.
    ///
    /// If this mutator is misconfigured, has no more mutations to perform, or
    /// otherwise cannot mutate `value` an error is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # fn foo() -> mutatis::Result<()> {
    /// use mutatis::{mutators as m, MutationContext, Mutator};
    ///
    /// // Define a mutator.
    /// let mut mutator = m::u8();
    ///
    /// // Mutate a value a bunch of times!
    /// let mut x = 100;
    /// let mut context = MutationContext::default();
    /// for _ in 0..5 {
    ///     mutator.mutate(&mut context, &mut x)?;
    ///     println!("mutated x is {x}");
    /// }
    ///
    /// // Example output:
    /// //
    /// //     mutated x is 196
    /// //     mutated x is 84
    /// //     mutated x is 162
    /// //     mutated x is 205
    /// //     mutated x is 233
    /// # Ok(())
    /// # }
    /// # foo().unwrap();
    /// ```
    fn mutate(&mut self, context: &mut MutationContext, value: &mut T) -> Result<()>;

    // Provided methods.

    /// Map a function over the mutations produced by this mutator.
    ///
    /// # Example
    ///
    /// ```
    /// # fn foo() -> mutatis::Result<()> {
    /// use mutatis::{mutators as m, MutationContext, Mutator};
    ///
    /// let mut context = MutationContext::default();
    ///
    /// let mut mutator = m::i32().map_mutate(|context, value| {
    ///     // Ensure that the value is always positive.
    ///     if *value <= 0 {
    ///         *value = context.rng().gen_u16() as i32;
    ///     }
    ///     Ok(())
    /// });
    ///
    /// let mut value = -42;
    ///
    /// for _ in 0..10 {
    ///     mutator.mutate(&mut context, &mut value)?;
    ///     assert!(value > 0, "the mutated value is always positive");
    /// }
    /// # Ok(())
    /// # }
    /// # foo().unwrap()
    /// ```
    #[inline]
    #[must_use = "mutator combinators do nothing until you call their `mutate` method"]
    fn map_mutate<F>(self, f: F) -> mutators::MapMutate<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut MutationContext, &mut T) -> Result<()>,
    {
        mutators::MapMutate { mutator: self, f }
    }

    /// Filter the mutations produced by this mutator.
    ///
    /// In general, you should prefer to use `map_mutate` to modify the value
    /// produced by a mutator, rather than `filter_mutate` to prevent certain
    /// values from being produced. Doing so will spend less time generating
    /// values that will be thrown away, giving you more time to actually work
    /// with the mutated values. That said, `filter_mutate` does work in a
    /// pinch.
    ///
    /// # Example
    ///
    /// ```
    /// # fn foo() -> mutatis::Result<()> {
    /// use mutatis::{mutators as m, MutationContext, Mutator};
    ///
    /// let mut context = MutationContext::default();
    ///
    /// let mut mutator = m::i32().filter_mutate(|context, value| {
    ///     // Only allow positive values.
    ///     *value > 0
    /// });
    ///
    /// let mut value = -42;
    ///
    /// for _ in 0..10 {
    ///     mutator.mutate(&mut context, &mut value)?;
    ///     assert!(value > 0, "the mutated value is always positive");
    /// }
    /// # Ok(())
    /// # }
    /// # foo().unwrap()
    /// ```
    #[inline]
    #[must_use = "mutator combinators do nothing until you call their `mutate` method"]
    fn filter_mutate<F>(self, f: F) -> mutators::FilterMutate<Self, F>
    where
        Self: Sized,
        F: FnMut(&MutationContext, &T) -> bool,
    {
        mutators::FilterMutate { mutator: self, f }
    }

    /// Run `n` consecutive mutations on a value for each call to `mutate`.
    ///
    /// # Example
    ///
    /// ```
    /// #![cfg(feature = "std")]
    /// # fn foo() -> mutatis::Result<()> {
    /// use mutatis::{mutators as m, MutationContext, Mutator};
    ///
    /// let mut context = MutationContext::default();
    ///
    /// let mut counter = 0;
    ///
    /// let mut mutator = m::i32()
    ///     .map_mutate(|_context, _value| {
    ///        counter += 1;
    ///        Ok(())
    ///     })
    ///     .mutate_n(3);
    ///
    /// let mut value = 42;
    ///
    /// mutator.mutate(&mut context, &mut value)?;
    /// assert_eq!(counter, 3, "the mutator was invoked 3 times");
    /// # Ok(())
    /// # }
    /// # foo().unwrap()
    /// ```
    #[inline]
    #[must_use = "mutator combinators do nothing until you call their `mutate` method"]
    fn mutate_n(self, n: u32) -> mutators::MutateN<Self>
    where
        Self: Sized,
    {
        mutators::MutateN { mutator: self, n }
    }

    /// Given a projection function `F: FnMut(&mut U) -> &mut T`, turn this
    /// `Mutator<T>` into a `Mutator<U>`.
    ///
    /// # Example
    ///
    /// ```
    /// use mutatis::{mutators as m, MutationContext, Mutator};
    /// # fn foo() -> mutatis::Result<()> {
    ///
    /// #[derive(Debug)]
    /// pub struct NewType(u32);
    ///
    /// let mut value = NewType(0);
    ///
    /// let mut mutator = m::u32().proj(|x: &mut NewType| &mut x.0);
    /// mutator.mutate(&mut MutationContext::default(), &mut value)?;
    ///
    /// println!("mutated value is {value:?}");
    /// // Example output:
    /// //
    /// //     mutated value is NewType(1682887620)
    /// # Ok(())
    /// # }
    /// # foo().unwrap()
    /// ```
    #[inline]
    #[must_use = "mutator combinators do nothing until you call their `mutate` method"]
    fn proj<F, U>(self, f: F) -> mutators::Proj<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut U) -> &mut T,
    {
        mutators::Proj { mutator: self, f }
    }

    /// Borrows a mutator, rather than consuming it.
    ///
    /// This is useful to allow applying mutator adapters while still retaining
    /// ownership of the original mutator.
    ///
    /// # Example
    ///
    /// ```
    /// use mutatis::{mutators as m, MutationContext, Mutator};
    /// # fn foo() -> mutatis::Result<()> {
    ///
    /// let mut mutator = m::u32().map_mutate(|_context, x| {
    ///     *x = x.wrapping_mul(4);
    ///     Ok(())
    /// });
    ///
    ///
    /// let mut value = 1234;
    /// let mut context = MutationContext::default();
    ///
    /// {
    ///     let mut borrowed_mutator = mutator.by_ref().map_mutate(|_context, x| {
    ///         *x = x.wrapping_add(1);
    ///         Ok(())
    ///     });
    ///     borrowed_mutator.mutate(&mut context, &mut value)?;
    ///     println!("first mutated value is {value}");
    /// }
    ///
    /// // In the outer scope, we can still use the original mutator.
    /// mutator.mutate(&mut context, &mut value)?;
    /// println!("second mutated value is {value}");
    ///
    /// // Example output:
    /// //
    /// //     first mutated value is 3665658153
    /// //     second mutated value is 42036549
    /// # Ok(())
    /// # }
    /// # foo().unwrap();
    #[inline]
    #[must_use = "mutator combinators do nothing until you call their `mutate` method"]
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

fn _static_assert_mutator_trait_is_object_safe(_: &dyn Mutator<u8>) {}

impl<M, T> Mutator<T> for &mut M
where
    M: Mutator<T>,
{
    fn mutate(&mut self, context: &mut MutationContext, value: &mut T) -> Result<()> {
        (**self).mutate(context, value)
    }
}

/// A trait for types that have a default mutator.
pub trait DefaultMutator {
    /// The default mutator for this type.
    type DefaultMutator: Mutator<Self> + Default;
}

/// A marker trait for mutators that will run a finite number of times and then
/// return an `ErrorKind::MutatorExhausted` error.
///
/// This is in contrast to mutators that may continue to run indefinitely,
/// yielding duplicate mutations along the way.
pub trait FusedMutator<T>: Mutator<T> {}

/// A mutator that can also generate a value from scratch.
pub trait GenerativeMutator<T>: Mutator<T> {
    /// Generate a random `T` value from scratch.
    ///
    /// Implementations may use the `context`'s random number generator in the
    /// process of generating a `T`.
    fn generate(&mut self, context: &mut MutationContext) -> Result<T>;
}

/// A mutator that supports clamping mutated values to within a given range.
///
/// To use `RangeMutator` implementations, use the
/// [`mutators::range()`][crate::mutators::range] function.
pub trait RangeMutator<T>: Mutator<T> {
    /// Mutate a value, ensuring that the resulting mutation is within the given
    /// range.
    fn mutate_in_range(
        &mut self,
        context: &mut MutationContext,
        value: &mut T,
        range: &ops::RangeInclusive<T>,
    ) -> Result<()>;
}
