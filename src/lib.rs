#![doc = include_str!("../README.md")]
#![no_std]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

mod log;

mod error;
mod rng;

use core::ops;

pub use error::*;
pub use rng::Rng;

#[cfg(feature = "check")]
pub mod check;
pub mod mutators;

#[cfg(feature = "derive")]
pub use mutatis_derive::Mutate;

/// A builder for configuring a mutation session.
///
/// This type allows you to configure things like setting the RNG seed, or
/// whether to only perform shrinking mutations.
///
/// Create a new builder via the [`MutationBuilder::new()`] method.
///
/// # Example
///
/// ```
/// use mutatis::MutationBuilder;
///
/// let context = MutationBuilder::new()
///     // Configure the RNG seed, changing which random mutation a mutator
///     // chooses to make.
///     .seed(0x12345678)
///     // Only perform mutations that "shrink" the test case.
///     .shrink(true);
/// ```
#[derive(Clone, Debug)]
pub struct MutationBuilder {
    context: MutationContext,
}

impl Default for MutationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MutationBuilder {
    /// Create a new, default `MutationBuilder`.
    pub fn new() -> Self {
        Self {
            context: MutationContext {
                rng: Rng::default(),
                shrink: false,
            },
        }
    }

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

    /// Mutate the given `value` with its default mutator and within the
    /// constraints of this `MutationBuilder`'s configuration.
    ///
    /// The default mutator for a type is defined by the [`DefaultMutate`] trait
    /// implementation for that type.
    ///
    /// To use a custom mutator, rather than the default mutator, use the
    /// [`mutate_with`] method instead.
    ///
    /// # Example
    ///
    /// ```
    /// # fn foo() -> mutatis::Result<()> {
    /// use mutatis::MutationBuilder;
    ///
    /// let mut x = Some(1234i32);
    ///
    /// let mut mtn = MutationBuilder::new().seed(0xaabbccdd);
    ///
    /// for _ in 0..5 {
    ///     mtn.mutate(&mut x)?;
    ///     println!("mutated x is {x:?}");
    /// }
    ///
    /// // Example output:
    /// //
    /// //     mutated x is None
    /// //     mutated x is Some(-688796504)
    /// //     mutated x is None
    /// //     mutated x is Some(-13390771)
    /// //     mutated x is Some(1208312368)
    /// # Ok(())
    /// # }
    /// # foo().unwrap();
    /// ```
    pub fn mutate<T>(&mut self, value: &mut T) -> Result<()>
    where
        T: DefaultMutate,
    {
        self.context.mutate(value)
    }

    /// Mutate the given `value` with the given `mutator` and within the
    /// constraints of this `MutationBuilder`'s configuration.
    ///
    /// This is similar to the [`mutate`] method, but allows you to specify a
    /// custom mutator to use instead of the default mutator for `value`'s type.
    ///
    /// # Example
    ///
    /// ```
    /// # fn foo() -> mutatis::Result<()> {
    /// use mutatis::{mutators as m, MutationBuilder};
    ///
    /// let mut res = Ok(1234i32);
    ///
    /// // Create a custom mutator for `Result<i32, bool>` values.
    /// let mut mutator = m::result(m::range(-10..=10), m::just(true));
    ///
    /// let mut mtn = MutationBuilder::new().seed(0x1984);
    ///
    /// for _ in 0..5 {
    ///     mtn.mutate_with(&mut mutator, &mut res)?;
    ///     println!("mutated res is {res:?}");
    /// }
    ///
    /// // Example output:
    /// //
    /// //     mutated res is Err(true)
    /// //     mutated res is Err(true)
    /// //     mutated res is Ok(9)
    /// //     mutated res is Err(true)
    /// //     mutated res is Ok(-6)
    /// # Ok(())
    /// # }
    /// # foo().unwrap();
    /// ```
    pub fn mutate_with<T>(&mut self, mutator: &mut impl Mutate<T>, value: &mut T) -> Result<()> {
        self.context.mutate_with(mutator, value)
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
/// New contexts are created via getting a [`MutationBuilder`] from the
/// [`MutationContext::builder()`] method, configuring the builder, and then
/// calling its [`build()`][MutationBuilder::build] method to get the
/// newly-created context. See the documentation for [`MutationBuilder`]
/// for an example of building and configuring a new context.
#[derive(Clone, Debug)]
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
    /// See the documentation for [`MutationBuilder`] for example usage.
    #[inline]
    #[must_use]
    pub fn builder() -> MutationBuilder {
        MutationBuilder::default()
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

    #[inline]
    pub(crate) fn mutate<T>(&mut self, value: &mut T) -> Result<()>
    where
        T: DefaultMutate,
    {
        let mut mutator = mutators::default::<T>();
        self.mutate_with(&mut mutator, value)
    }

    #[inline]
    pub(crate) fn mutate_with<T>(
        &mut self,
        mutator: &mut impl Mutate<T>,
        value: &mut T,
    ) -> Result<()> {
        self.choose_and_apply_mutation(value, |muts, value| mutator.mutate(muts, value))
    }

    fn choose_and_apply_mutation<T>(
        &mut self,
        value: &mut T,
        mut mutate_impl: impl FnMut(&mut MutationSet, &mut T) -> Result<()>,
    ) -> Result<()> {
        log::trace!("=== choosing an applying a mutation ===");

        // Count how many mutations we *could* perform.
        let mut muts = MutationSet {
            context: self,
            phase: Phase::Count(0),
            applied_mutation: false,
        };
        mutate_impl(&mut muts, value)?;

        let count = match muts.phase {
            Phase::Count(count) => usize::try_from(count).unwrap(),
            Phase::Mutate { .. } => unreachable!(),
        };
        log::trace!("counted {count} mutations");

        if count == 0 {
            log::trace!("mutator exhausted");
            return Err(Error::exhausted());
        }

        // Choose a random target mutation to actually perform.
        let target = muts.context.rng().gen_index(count).unwrap();
        log::trace!("targeting mutation {target}");
        debug_assert!(target < count);

        // Perform the chosen target mutation.
        muts.phase = Phase::Mutate {
            current: 0,
            target: u32::try_from(target).unwrap(),
        };
        match mutate_impl(&mut muts, value) {
            Err(e) if e.is_early_exit() => {
                log::trace!("mutation applied successfully");
                Ok(())
            }

            Err(e) => {
                log::error!("failed to apply mutation: {e}");
                Err(e)
            }

            // We should have found the target mutation, applied it, and then
            // broken out of the mutation loop by returning
            // `Err(Error::early_exit())`. So either we are facing a
            // nondeterministic mutations enumeration or a mutator is missing a
            // `?` and is failing to propagate the early-exit error to
            // us. Differentiate between these two cases via the
            // `applied_mutation` flag.
            Ok(()) if muts.applied_mutation => {
                panic!(
                    "We applied a mutation but did not receive an early-exit error \
                     from the mutator. This means that errors are not always being \
                     propagated, for example a `?` is missing from a call to the \
                     `MutationSet::mutation` method. Errors must be propagated \
                     in `Mutate::mutate` et al method implementations; failure to do \
                     so can lead to bugs, panics, and degraded performance.",
                )
            }
            Ok(()) => {
                let current = match muts.phase {
                    Phase::Mutate { current, .. } => current,
                    _ => unreachable!(),
                };
                panic!(
                    "Nondeterministic mutator implementation: did not enumerate the \
                     same set of mutations when given the same value! Counted {count} \
                     mutations in the first pass, but only found {current} mutations on \
                     the second pass. Mutators must be deterministic.",
                )
            }
        }
    }

    #[inline]
    pub(crate) fn mutate_in_range_with<T>(
        &mut self,
        mutator: &mut impl MutateInRange<T>,
        value: &mut T,
        range: &ops::RangeInclusive<T>,
    ) -> Result<()> {
        self.choose_and_apply_mutation(value, |muts, value| {
            mutator.mutate_in_range(muts, value, range)
        })
    }
}

#[derive(Clone, Copy)]
enum Phase {
    Count(u32),
    Mutate { current: u32, target: u32 },
}

/// The set of mutations that can be applied to a value.
///
/// This type is used by mutators to register the mutations that they can
/// perform on a value. It is passed to the [`Mutate::mutate`] trait method, and
/// provides a way to register candidate mutations, as well as to check if
/// shrinking is enabled.
pub struct MutationSet<'a> {
    context: &'a mut MutationContext,
    phase: Phase,
    applied_mutation: bool,
}

impl<'a> MutationSet<'a> {
    /// Register a candidate mutation that can be applied to a value.
    ///
    /// This method is called by [`Mutate::mutate`] implementations to register
    /// the potential mutations that they can perform on a value.
    ///
    /// `f` should be a closure that performs the mutation on the value that was
    /// passed to `Mutate::mutate`, updating the value and the mutator itself as
    /// necessary.
    ///
    /// See the [`Mutate::mutate`] trait method documentation for more
    /// information on this method's use.
    #[inline]
    pub fn mutation(
        &mut self,
        mut f: impl FnMut(&mut MutationContext) -> Result<()>,
    ) -> Result<()> {
        match &mut self.phase {
            Phase::Count(count) => {
                *count += 1;
                Ok(())
            }
            Phase::Mutate { current, target } => {
                assert!(
                    *current <= *target,
                    "{current} <= {target}; did you forget to `?`-propagate the \
                     result of a `MutationSet::mutation` call?",
                );
                if *current == *target {
                    self.applied_mutation = true;
                    f(&mut self.context)?;
                    Err(Error::early_exit())
                } else {
                    *current += 1;
                    Ok(())
                }
            }
        }
    }

    /// Whether only shrinking mutations should be registered in this mutation
    /// set or not.
    ///
    /// When this method returns `true`, then you should not register any
    /// mutation which can grow the value being mutated.
    pub fn shrink(&self) -> bool {
        self.context.shrink()
    }
}

/// A trait for mutating values.
///
/// You can think of `Mutate<T>` as a streaming iterator of `T`s but instead of
/// internally containing and yielding access to the `T`s, it takes an `&mut T`
/// as an argument and mutates it in place.
///
/// The main method is the [`mutate`][Mutate::mutate] method, which applies one
/// of many potential mutations to the given value, or returns an error.
///
/// # Example: Using a Type's Default Mutator
///
/// Many types implement the `DefaultMutate` trait, which provides a default
/// mutator for that type. You can use this default mutator by calling
/// [`mutate`][MutationBuilder::mutate] on a `MutationBuilder` with a value of
/// that type.
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// # #![cfg(feature = "std")]
/// use mutatis::{MutationContext, MutationBuilder};
///
/// let mut builder = MutationBuilder::new();
///
/// let mut x = 1234;
/// builder.mutate(&mut x)?;
///
/// for _ in 0..5 {
///     builder.mutate(&mut x)?;
///     println!("mutated x is {x}");
/// }
///
/// panic!();
/// // Example output:
/// //
/// //     mutated x is 1682887620
/// # Ok(())
/// # }
/// ```
///
/// # Example: Using Custom Mutators
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// # #![cfg(feature = "std")]
/// use mutatis::{mutators as m, Mutate, MutationBuilder};
///
/// // Define a mutator for `u32`s that only creates multiples-of-four
/// let mut mutator = m::u32()
///     .map(|_ctx, x| {
///         *x = *x & !3; // Clear the bottom two bits to make `x` a multiple of four.
///         Ok(())
///     });
///
/// // Mutate a value a bunch of times!
/// let mut x = 1234;
/// let mut builder = MutationBuilder::new();
/// for _ in 0..5 {
///     builder.mutate_with(&mut mutator, &mut x)?;
///     println!("mutated x is {x}");
/// }
///
/// panic!();
/// // Example output:
/// //
/// //     mutated x is 2436583184
/// //     mutated x is 2032949584
/// //     mutated x is 2631247496
/// //     mutated x is 199875380
/// //     mutated x is 3751781284
/// # Ok(())
/// # }
/// ```
///
/// # Exhaustion
///
/// A mutator may become *exhausted*, meaning that it doesn't have any more
/// mutations it can perform for a given value. In this case, the mutator may
/// return an error of kind [`ErrorKind::Exhausted`]. Many mutators are
/// effectively inexhaustible (or it would be prohibitively expensive to
/// precisely track whether they've emitted every possible mutation of a value,
/// like a mutator that flips a single bit in a `u32`) and therefore it is valid
/// for a mutator to never report exhaustion.
///
/// You may also ignore exhaustion errors via the
/// [`ResultExt::ignore_exhausted`] extension method.
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
/// # fn foo () {
/// # #![cfg(feature = "derive")]
/// use mutatis::{
///     mutators as m, DefaultMutate, Mutate, MutationBuilder, MutationSet,
///     Result,
/// };
///
/// #[derive(Mutate)] // Derive a default mutator.
/// pub struct Foo(u32);
///
/// // Define and implement a second mutator type for `Foo` by hand!
///
/// pub struct AlignedFooMutator{
///     inner: <Foo as DefaultMutate>::DefaultMutate,
///     alignment: u32,
/// }
///
/// impl Mutate<Foo> for AlignedFooMutator {
///     fn mutate(&mut self, mutations: &mut MutationSet, foo: &mut Foo) -> Result<()> {
///         self.inner
///             .by_ref()
///             .map(|_context, foo| {
///                 // Clear the bottom bits to keep the `Foo` "aligned".
///                 debug_assert!(self.alignment.is_power_of_two());
///                 let mask = !(self.alignment - 1);
///                 foo.0 = foo.0 & mask;
///                 Ok(())
///             })
///             .mutate(mutations, foo)
///     }
/// }
/// # }
/// ```
pub trait Mutate<T>
where
    T: ?Sized,
{
    // Required methods.

    /// Pseudo-randomly mutate the given value.
    ///
    /// # Calling the `mutate` Method
    ///
    /// If you just want to mutate a value, use [`MutationBuilder::mutate`] or
    /// [`MutationBuilder::mutate_with`] instead of invoking this trait method
    /// directly. See their documentation for more details.
    ///
    /// # Implementing the `mutate` Method
    ///
    /// Register every mutation that a mutator *could* perform by invoking the
    /// [`mutations.mutation(...)`][MutationSet::mutation] function, passing in
    /// a closure that performs that mutation, updating `value` and `self` as
    /// necessary.
    ///
    /// `mutate` implementations must only mutate `self` and the given `value`
    /// from inside a registered mutation closure. It must not update `self` or
    /// modify `value` outside of one of those mutation closures.
    ///
    /// Furthermore, all `mutate` implementations must be deterministic: given
    /// the same inputs, the same set of mutations must be registered in the
    /// same order.
    ///
    /// These requirements exist because, under the hood, the `mutate` method is
    /// called twice for every mutation that is actually performed:
    ///
    /// 1. First, `mutate` is called to count all the possible mutations that
    ///    could be performed. In this phase, the mutation closures are ignored.
    ///
    /// 2. Next, a random index `i` between `0` and that count is chosen. This
    ///    is the index of the mutation that we will actually be applied.
    ///
    /// 3. Finally, `mutate` is called again. In this phase, the `i`th mutation
    ///    closure is invoked, applying the mutation, while all others are
    ///    ignored.
    ///
    /// Note that the registered mutations are roughly uniformly selected from,
    /// so if you wish to skew the distribution of mutations, making certain
    /// mutations more probable than others, you may register mutations multiple
    /// times or register overlapping mutations.
    ///
    /// ## Example
    ///
    /// ```
    /// # fn foo() -> mutatis::Result<()> {
    /// use mutatis::{
    ///     mutators as m, Generate, Mutate, MutationBuilder, MutationSet,
    ///     Result,
    /// };
    ///
    /// // A custom mutator that creates pairs where the first element is less
    /// // than or equal to the second.
    /// pub struct OrderedPairs;
    ///
    /// impl Mutate<(u64, u64)> for OrderedPairs {
    ///     fn mutate(
    ///         &mut self,
    ///         mutations: &mut MutationSet<'_>,
    ///         pair: &mut (u64, u64),
    ///     ) -> Result<()> {
    ///         // We *cannot* mutate `self` or `pair` out here.
    ///
    ///         if *pair != (0, 0) {
    ///             // Note: we register this mutation -- even when not
    ///             // shrinking and even though the subsequent mutation
    ///             // subsumes this one -- to bias the distribution towards
    ///             // smaller values.
    ///             mutations.mutation(|ctx| {
    ///                 // We *can* mutate `self` and `pair` inside here.
    ///                 let a = m::range(0..=pair.0).generate(ctx)?;
    ///                 let b = m::range(0..=pair.1).generate(ctx)?;
    ///                 *pair = (a.min(b), a.max(b));
    ///                 Ok(())
    ///             })?;
    ///         }
    ///
    ///         if !mutations.shrink() {
    ///             // Only register this fully-general mutation when we are
    ///             // not shrinking, as this can grow the pair.
    ///             mutations.mutation(|ctx| {
    ///                 // We *can* mutate `self` and `pair` inside here.
    ///                 let a = m::u64().generate(ctx)?;
    ///                 let b = m::u64().generate(ctx)?;
    ///                 *pair = (a.min(b), a.max(b));
    ///                 Ok(())
    ///             })?;
    ///         }
    ///
    ///         Ok(())
    ///     }
    /// }
    ///
    /// // Create a pair.
    /// let mut pair = (1000, 2000);
    ///
    /// // And mutate it a bunch of times!
    /// let mut mtn = MutationBuilder::new();
    /// for _ in 0..3 {
    ///     mtn.mutate_with(&mut OrderedPairs, &mut pair)?;
    ///     println!("mutated pair is {pair:?}");
    /// }
    ///
    /// // Example output:
    /// //
    /// //     mutated pair is (11, 861)
    /// //     mutated pair is (8, 818)
    /// //     mutated pair is (3305948426120559093, 16569598107406464568)
    /// # Ok(())
    /// # }
    /// # foo().unwrap();
    /// ```
    fn mutate(&mut self, mutations: &mut MutationSet<'_>, value: &mut T) -> Result<()>;

    // Provided methods.

    /// Create a new mutator that performs either this mutation or the `other`
    /// mutation.
    ///
    /// # Example
    ///
    /// ```
    /// # fn foo() -> mutatis::Result<()> {
    /// use mutatis::{mutators as m, Mutate, MutationBuilder};
    ///
    /// let mut mtn = MutationBuilder::new();
    ///
    /// // Either generate `-1`...
    /// let mut mutator = m::just(-1)
    ///     // ...or values in the range `0x40..=0x4f`...
    ///     .or(m::range(0x40..=0x4f))
    ///     // ...or values with just a single bit set.
    ///     .or(m::range(0..=31).map(|_ctx, x| {
    ///         *x = 1 << *x;
    ///         Ok(())
    ///     }));
    ///
    /// let mut value = 0;
    ///
    /// for _ in 0..5 {
    ///     mtn.mutate_with(&mut mutator, &mut value)?;
    ///     println!("mutated value is {value:#x}");
    /// }
    ///
    /// // Example output:
    /// //
    /// //     mutated value is 0x4a
    /// //     mutated value is 0xffffffff
    /// //     mutated value is 0x400000
    /// //     mutated value is 0x20000000
    /// //     mutated value is 0x4e
    /// # Ok(())
    /// # }
    /// # foo().unwrap()
    /// ```
    fn or<M>(self, other: M) -> mutators::Or<Self, M>
    where
        Self: Sized,
    {
        mutators::Or {
            left: self,
            right: other,
        }
    }

    /// Map a function over the mutations produced by this mutator.
    ///
    /// # Example
    ///
    /// ```
    /// # fn foo() -> mutatis::Result<()> {
    /// use mutatis::{mutators as m, Mutate, MutationBuilder};
    ///
    /// let mut mtn = MutationBuilder::new();
    ///
    /// let mut mutator = m::i32().map(|context, value| {
    ///     // Ensure that the value is always positive.
    ///     if *value <= 0 {
    ///         *value = i32::from(context.rng().gen_u16());
    ///     }
    ///     Ok(())
    /// });
    ///
    /// let mut value = -42;
    ///
    /// for _ in 0..10 {
    ///     mtn.mutate_with(&mut mutator, &mut value)?;
    ///     assert!(value > 0, "the mutated value is always positive");
    /// }
    /// # Ok(())
    /// # }
    /// # foo().unwrap()
    /// ```
    #[inline]
    #[must_use = "mutator combinators do nothing until you call their `mutate` method"]
    fn map<F>(self, f: F) -> mutators::Map<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut MutationContext, &mut T) -> Result<()>,
    {
        mutators::Map { mutator: self, f }
    }

    /// Given a projection function `F: FnMut(&mut U) -> &mut T`, turn this
    /// `Mutate<T>` into a `Mutate<U>`.
    ///
    /// # Example
    ///
    /// ```
    /// use mutatis::{mutators as m, Mutate, MutationBuilder};
    /// # fn foo() -> mutatis::Result<()> {
    ///
    /// #[derive(Debug)]
    /// pub struct NewType(u32);
    ///
    /// let mut value = NewType(0);
    ///
    /// let mut mutator = m::u32().proj(|x: &mut NewType| &mut x.0);
    ///
    /// let mut mtn = MutationBuilder::new();
    /// for _ in 0..3 {
    ///    mtn.mutate_with(&mut mutator, &mut value)?;
    ///    println!("mutated value is {value:?}");
    /// }
    ///
    /// // Example output:
    /// //
    /// //     mutated value is NewType(3729462868)
    /// //     mutated value is NewType(49968845)
    /// //     mutated value is NewType(2440803355)
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
    /// use mutatis::{mutators as m, Mutate, MutationBuilder};
    /// # fn foo() -> mutatis::Result<()> {
    ///
    /// let mut mutator = m::u32().map(|_context, x| {
    ///     *x = *x & !3;
    ///     Ok(())
    /// });
    ///
    ///
    /// let mut value = 1234;
    /// let mut mtn = MutationBuilder::new();
    ///
    /// {
    ///     let mut borrowed_mutator = mutator.by_ref().map(|_context, x| {
    ///         *x = x.wrapping_add(1);
    ///         Ok(())
    ///     });
    ///     mtn.mutate_with(&mut borrowed_mutator, &mut value)?;
    ///     println!("first mutated value is {value}");
    /// }
    ///
    /// // In the outer scope, we can still use the original mutator.
    /// mtn.mutate_with(&mut mutator, &mut value)?;
    /// println!("second mutated value is {value}");
    ///
    /// // Example output:
    /// //
    /// //     first mutated value is 3729462869
    /// //     second mutated value is 49968844
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

fn _static_assert_object_safety(
    _: &dyn Mutate<u8>,
    _: &dyn Generate<u8>,
    _: &dyn MutateInRange<u8>,
) {
}

impl<M, T> Mutate<T> for &mut M
where
    M: Mutate<T>,
{
    fn mutate(&mut self, muts: &mut MutationSet, value: &mut T) -> Result<()> {
        (**self).mutate(muts, value)
    }
}

/// A trait for types that have a default mutator.
pub trait DefaultMutate {
    /// The default mutator for this type.
    type DefaultMutate: Mutate<Self> + Default;
}

/// A mutator that can also generate a value from scratch.
pub trait Generate<T>: Mutate<T> {
    /// Generate a random `T` value from scratch.
    ///
    /// Implementations may use the `context`'s random number generator in the
    /// process of generating a `T`.
    fn generate(&mut self, context: &mut MutationContext) -> Result<T>;
}

/// A mutator that supports clamping mutated values to within a given range.
///
/// To use `MutateInRange` implementations, use the
/// `[MutationBuilder::mutate_in_range]` method,
/// `[MutationBuilder::mutate_in_range_with]` method, or
/// [`mutators::range()`][crate::mutators::range] combinator.
pub trait MutateInRange<T>: Mutate<T> {
    /// Mutate a value, ensuring that the resulting mutation is within the given
    /// range.
    fn mutate_in_range(
        &mut self,
        mutations: &mut MutationSet,
        value: &mut T,
        range: &ops::RangeInclusive<T>,
    ) -> Result<()>;
}
