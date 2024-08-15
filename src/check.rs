//! A small framework for property-based testing with `mutatis::Mutator`.
//!
//! The entry point for this framework is the [`Check`] type.
//!
//! This framework is designed to be used for smoke tests inside `#[test]`
//! functions. It should compile and run quickly, and is therefore suitable for
//! quick (but relatively shallow) iteration cycles like `cargo test` runs and
//! CI. It is not intended to be used for your main, in-depth, 24/7 fuzzing. For
//! that use case, integrate `mutatis` into a more fully-featured,
//! coverage-guided, mutation-based framework, such as `libfuzzer`.
//!
//! Rather than writing unit tests that make assertions based on some finite
//! number of inputs, and completely separately authoring a fuzzer or two,
//! consider doing this instead:
//!
//! * Write a few functions that assert that a particular property or invariant
//!   is maintained when given a single input.
//!
//! * Use [`mutatis::check::Check`][Check] to turn those functions into
//!   property-based tests, providing the finite inputs you would have used in
//!   the unit tests as an initial corpus.
//!
//! * Use those functions as fuzzing oracles with `libfuzzer`, or another
//!   coverage-guided fuzzing engine.
//!
//! Now your `cargo test`s are exercising not only the subset of the state space
//! you explicitly described in those finite inputs, but also everything
//! reachable from mutating those inputs (bounded by the configured number of
//! check iterations). Additionally, you can assert these same properties and
//! invariants are upheld 24/7 with a coverage-guided fuzzer like `libfuzzer` to
//! explore the state space even more thoroughly.
//!
//! # Example
//!
//! ```
//! #[cfg(test)]
//! mod tests {
//!     use mutatis::{check::Check, mutators as m};
//!
//!     #[test]
//!     fn test_rgb_to_hsl_to_rgb_round_trip() {
//!         let result = Check::new()
//!             // Check the property on 1000 mutated values.
//!             .iters(1000)
//!             // If we find a failing test case, try to shrink it down to a
//!             // minimal failing test case by running 1000 shrink iterations.
//!             .shrink_iters(1000)
//!             // Run the property check!
//!             .run(
//!                 // The mutator we'll use to generate new values.
//!                 m::array(m::range(0..=0xff)),
//!                 // The initial corpus of values to check and to derive new
//!                 // inputs from via mutation.
//!                 [
//!                     [0x00, 0x00, 0x00],
//!                     [0xff, 0xff, 0xff],
//!                     [0x66, 0x33, 0x99],
//!                 ],
//!                 // The property to check: RGB -> HSL -> RGB should be the
//!                 // identity function.
//!                 |[r, g, b]| {
//!                     let [h, s, l] = rgb_to_hsl(*r, *g, *b);
//!                     let [r2, g2, b2] = hsl_to_rgb(h, s, l);
//!                     if [*r, *g, *b] == [r2, g2, b2] {
//!                         Ok(())
//!                     } else {
//!                         Err("round-trip conversion failed!")
//!                     }
//!                 },
//!             );
//!         assert!(result.is_ok());
//!     }
//! # fn rgb_to_hsl(r: u8, g: u8, b: u8) -> [u8; 3] { [0, 0, 0] }
//! # fn hsl_to_rgb(h: u8, s: u8, l: u8) -> [u8; 3] { [0, 0, 0] }
//! }
//! ```

use super::*;
use crate::mutators as m;
use std::fmt::{self, Debug};
use std::panic;
use std::prelude::v1::*;

/// The result of running a check.
///
/// If the check passes, this is `Ok(())`.
///
/// If the check fails, this is `Err(CheckError::Failed(_))` with the failing
/// test case and an error message.
///
/// If there is some other kind of error while running the check, for example if
/// a `Mutator` does not support the given `MutationContext` configuration, then
/// this is `Err(CheckError::Error(_))`.
pub type CheckResult<T> = std::result::Result<(), CheckError<T>>;

/// An error when running a `Check`.
pub enum CheckError<T> {
    /// The check failed.
    ///
    /// This indicates that the property being checked is not upheld for the
    /// given test case.
    Failed(CheckFailure<T>),

    /// The corpus was empty.
    EmptyCorpus,

    /// An error occurred while running the check.
    MutatorError(Error),
}

impl<T> From<Error> for CheckError<T> {
    fn from(v: Error) -> Self {
        Self::MutatorError(v)
    }
}

impl<T> From<CheckFailure<T>> for CheckError<T> {
    fn from(v: CheckFailure<T>) -> Self {
        Self::Failed(v)
    }
}

impl<T> std::error::Error for CheckError<T>
where
    T: 'static + Debug,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CheckError::MutatorError(e) => Some(e),
            CheckError::Failed(f) => Some(f),
            CheckError::EmptyCorpus => None,
        }
    }
}

impl<T> fmt::Display for CheckError<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckError::Failed(e) => write!(f, "check failure: {e}"),
            CheckError::EmptyCorpus => write!(f, "cannot check an empty corpus"),
            CheckError::MutatorError(e) => write!(f, "mutator error: {e}"),
        }
    }
}

impl<T> Debug for CheckError<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<T> CheckError<T> {
    /// Unwrap the underlying `CheckError::Failed(_)` payload, panicking if this
    /// is not a `CheckError::Failed`.
    #[track_caller]
    pub fn unwrap_failed(self) -> CheckFailure<T> {
        match self {
            CheckError::Failed(f) => f,
            _ => panic!("CheckError::unwrap_failed called on non-failed CheckError"),
        }
    }

    /// Unwrap the underlying `CheckError::MutatorError(_)` payload, panicking
    /// if this is not a `CheckError::MutatorError(_)`.
    #[track_caller]
    pub fn unwrap_mutator_error(self) -> Error {
        match self {
            CheckError::MutatorError(e) => e,
            _ => panic!("CheckError::unwrap_error called on non-error CheckError"),
        }
    }
}

/// An error that occurred while running a check.
///
/// This contains the failing test case and a message describing the failure.
///
/// # Example
///
/// ```
/// use mutatis::{check::Check, mutators as m};
///
/// let failure = Check::new()
///     .run(
///         m::default::<bool>(),
///         [true],
///         |b| {
///             if *b {
///                 Ok(())
///             } else {
///                 Err("expected true!")
///             }
///         },
///     )
///     .unwrap_err()
///     .unwrap_failed();
///
/// assert_eq!(failure.value, false);
/// assert_eq!(failure.message, "expected true!");
/// ```
#[non_exhaustive]
pub struct CheckFailure<T> {
    /// The input value that triggered the failure.
    pub value: T,

    /// The failure message.
    pub message: String,
}

impl<T> fmt::Display for CheckFailure<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let CheckFailure { value, message } = self;
        write!(f, "failed on input {value:?}: {message}")
    }
}

impl<T> Debug for CheckFailure<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<T> std::error::Error for CheckFailure<T> where T: Debug {}

/// A check that can be run to test a property.
///
/// See [the module-level documentation][crate::check] for example usage.
#[derive(Debug)]
pub struct Check {
    iters: usize,
    shrink_iters: usize,
}

impl Default for Check {
    fn default() -> Check {
        Check::new()
    }
}

impl Check {
    /// Create a new `Check`.
    pub fn new() -> Check {
        Check {
            iters: 1000,
            shrink_iters: 1000,
        }
    }

    /// Configure the number of test iterations to run.
    pub fn iters(&mut self, iters: usize) -> &mut Check {
        self.iters = iters;
        self
    }

    /// Configure the number of attempts to shrink a failing input before
    /// reporting the failure.
    pub fn shrink_iters(&mut self, shrink_iters: usize) -> &mut Check {
        self.shrink_iters = shrink_iters;
        self
    }

    /// Run this configured `Check` with a default initial `T` value and
    /// mutator.
    ///
    /// This is a convenience method that is equivalent to calling
    /// [`run`][Check::run] with [`m::default::<T>()`][crate::mutators::default]
    /// and [`T::default()`][core::default::Default::default] as the only value
    /// in the initial corpus.
    pub fn run_with_defaults<T, S>(
        &self,
        property: impl FnMut(&T) -> std::result::Result<(), S>,
    ) -> CheckResult<T>
    where
        T: Clone + Debug + Default + DefaultMutator,
        S: ToString,
    {
        self.run(m::default::<T>(), [T::default()], property)
    }

    /// Run this configured `Check`.
    ///
    /// The `initial_corpus` is used to seed the check with some initial
    /// values. If you have some known edge cases that you want to test, you can
    /// provide them here. There must always be at least one value in the
    /// `initial_corpus`, otherwise a [`CheckError::EmptyCorpus`] error is
    /// returned.
    ///
    /// All values in the `initial_corpus` are checked before we begin any
    /// mutation iterations.
    ///
    /// The `mutator` is used to generate new `T` values from existing ones in
    /// the given corpus.
    ///
    /// The `property` is the function that is called for each value in the
    /// corpus and for each mutated value. If the property returns an error, the
    /// check is considered to have failed and the failing value is shrunk down
    /// to a minimal failing value. You can configure how much effor is put into
    /// shrinking via the [`shrink_iters`][Check::shrink_iters] method.
    pub fn run<M, T, S>(
        &self,
        mut mutator: M,
        initial_corpus: impl IntoIterator<Item = T>,
        mut property: impl FnMut(&T) -> std::result::Result<(), S>,
    ) -> CheckResult<T>
    where
        M: Mutator<T>,
        T: Clone + Debug,
        S: ToString,
    {
        let mut corpus = initial_corpus.into_iter().collect::<Vec<_>>();
        if corpus.is_empty() {
            return Err(CheckError::EmptyCorpus);
        }

        // First, double check that the property is maintained for all values in
        // the initial corpus.
        for value in &corpus {
            if let Err(msg) = Self::check_one(value, &mut property) {
                return self.shrink(mutator, value.clone(), property, msg);
            }
        }

        // Second, run the check on mutated values derived from the corpus for
        // the configured iterations.
        let mut context = MutationContext::default();
        for _ in 0..self.iters {
            let index = context.rng().gen_index(corpus.len()).unwrap();

            match mutator.mutate(&mut context, &mut corpus[index]) {
                Ok(()) => {}
                Err(e) if e.is_mutator_exhausted() => {
                    corpus.swap_remove(index);
                    if corpus.is_empty() {
                        return Ok(());
                    }
                }
                Err(e) => return Err(e.into()),
            }

            if let Err(msg) = Self::check_one(&corpus[index], &mut property) {
                return self.shrink(mutator, corpus[index].clone(), property, msg);
            }
        }

        Ok(())
    }

    fn check_one<T, S>(
        value: &T,
        mut property: impl FnMut(&T) -> std::result::Result<(), S>,
    ) -> std::result::Result<(), String>
    where
        T: Debug,
        S: ToString,
    {
        match panic::catch_unwind(panic::AssertUnwindSafe(|| property(value))) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(msg)) => Err(msg.to_string()),
            Err(_) => Err("<panicked>".into()),
        }
    }

    fn shrink<M, T, S>(
        &self,
        mut mutator: M,
        mut value: T,
        mut property: impl FnMut(&T) -> std::result::Result<(), S>,
        mut message: String,
    ) -> CheckResult<T>
    where
        M: Mutator<T>,
        T: Clone + Debug,
        S: ToString,
    {
        log::warn!("failed on input {value:?}: {message}");
        if self.shrink_iters == 0 {
            return Err(CheckFailure { value, message }.into());
        }

        log::debug!("shrinking for {} iters...", self.shrink_iters);

        let mut context = MutationContext::builder().shrink(true).build();

        for _ in 0..self.shrink_iters {
            let mut candidate = value.clone();

            match mutator.mutate(&mut context, &mut candidate) {
                // If the mutator is exhausted, then don't keep trying to shrink
                // the input and just report the final error.
                Err(e) if e.is_mutator_exhausted() => break,

                // Ignore mutator errors during shrinking because it is more
                // important to report the property failure.
                Err(e) => {
                    log::info!("got mutator error during shrinking, ignoring: {e}");
                    continue;
                }

                Ok(()) => {}
            }

            match Self::check_one(&candidate, &mut property) {
                Ok(()) => {
                    // Not a failure, throw away this candidate and try another
                    // mutation.
                }
                Err(msg) => {
                    message = msg;
                    log::debug!("got failure for shrunken input {value:?}: {message}");
                    value = candidate;
                }
            }
        }

        log::info!("shrunk failing input down to {value:?}");
        Err(CheckFailure { value, message }.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::result::Result;

    fn check() -> Check {
        let _ = env_logger::builder().is_test(true).try_init();
        Check::new()
    }

    #[test]
    fn check_run_okay() {
        check()
            .run(m::just(true), [true], |b: &bool| {
                if *b {
                    Ok(())
                } else {
                    Err("expected true!")
                }
            })
            .unwrap();
    }

    #[test]
    fn check_run_fail() {
        let failure = check()
            .run(m::bool(), [true], |b: &bool| {
                if *b {
                    Ok(())
                } else {
                    Err("expected true!")
                }
            })
            .unwrap_err()
            .unwrap_failed();

        assert_eq!(failure.value, false);
        assert_eq!(failure.message, "expected true!");
    }

    #[test]
    fn check_run_empty_corpus() {
        let result = check().run(m::bool(), [], |b: &bool| {
            if *b {
                Ok(())
            } else {
                Err("expected true!")
            }
        });

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CheckError::EmptyCorpus));
    }

    #[test]
    fn check_run_fail_and_shrink() {
        let failure = check()
            .shrink_iters(1000)
            .run(m::u8(), [u8::MAX], |x: &u8| {
                if *x < 10 {
                    Ok(())
                } else {
                    Err("expected < 10")
                }
            })
            .unwrap_err()
            .unwrap_failed();

        assert_eq!(failure.value, 10);
        assert_eq!(failure.message, "expected < 10");
    }

    #[test]
    fn check_run_fail_on_panic() {
        let result = check().run(m::bool(), [true], |_: &bool| -> Result<(), String> {
            panic!("oh no!")
        });
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CheckError::Failed(_)));
    }

    #[test]
    fn check_run_fail_on_panic_and_shrink() {
        let failure = check()
            .shrink_iters(1000)
            .run(m::u8(), [u8::MAX], |x: &u8| -> Result<(), String> {
                assert!(*x < 10);
                Ok(())
            })
            .unwrap_err()
            .unwrap_failed();

        assert_eq!(failure.value, 10);
        assert_eq!(failure.message, "<panicked>");
    }
}
