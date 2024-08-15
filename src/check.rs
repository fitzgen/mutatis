//! A small framework for property-based testing with `mutatis::Mutator`.
//!
//! The primary entry point for this framework is the [`Check`] type.
//!
//! This framework is designed to be used for smoke tests inside `#[test]`
//! functions. It should compile and run quickly, and is therefore suitable for
//! quick (but relatively shallow) iteration cycles like `cargo test` runs and
//! CI. It is not intended to be used for your main, in-depth, 24/7 fuzzing. For
//! that use case, integrate `mutatis` into a more fully-featured,
//! coverage-guided, mutation-based framework, such as `libfuzzer`.
//!
//! # Example
//!
//! ```
//! mod tests {
//!     use mutatis::check::Check;
//!
//!     fn test_addition() {
//!         let result = Check::new()
//!             .iters(1000)
//!             .shrink_iters(1000)
//!             .run_with_defaults(|(a, b): &(i32, i32)| {
//!                 if a + b == b + a {
//!                     Ok(())
//!                 } else {
//!                     Err("addition is not commutative!")
//!                 }
//!             });
//!         assert!(result.is_ok());
//!     }
//! }
//! ```

use super::*;
use crate::mutators as m;
use std::fmt::Debug;
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

    /// An error occurred while running the check.
    Error(Error),
}

impl<T> From<Error> for CheckError<T> {
    fn from(v: Error) -> Self {
        Self::Error(v)
    }
}

impl<T> From<CheckFailure<T>> for CheckError<T> {
    fn from(v: CheckFailure<T>) -> Self {
        Self::Failed(v)
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

    /// Unwrap the underlying `CheckError::Error(_)` payload, panicking if this
    /// is not a `CheckError::Error(_)`.
    #[track_caller]
    pub fn unwrap_error(self) -> Error {
        match self {
            CheckError::Error(e) => e,
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
///         true,
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
#[derive(Debug)]
#[non_exhaustive]
pub struct CheckFailure<T> {
    /// The input value that triggered the failure.
    pub value: T,

    /// The failure message.
    pub message: String,
}

/// A check that can be run to test a property.
///
/// # Example
///
/// ```
/// mod tests {
///     use mutatis::{check::Check, mutators};
///
///     fn test_rgb_to_hsl_to_rgn_round_trip() {
///         let result = Check::new()
///             // Check the property on 1000 mutated values.
///             .iters(1000)
///             // If we find a failing test case, try to shrink it down to a
///             // minimal failing test case by running 1000 shrink iterations.
///             .shrink_iters(1000)
///             // Run the property check!
///             .run(
///                 mutators::array(mutators::range(0..=255)),
///                 [0x66, 0x33, 0x99],
///                 |[r, g, b]| {
///                     let [h, s, l] = rgb_to_hsl(*r, *g, *b);
///                     let [r2, g2, b2] = hsl_to_rgb(h, s, l);
///                     if [*r, *g, *b] == [r2, g2, b2] {
///                         Ok(())
///                     } else {
///                         Err("round-trip conversion failed!")
///                     }
///                 },
///             );
///         assert!(result.is_ok());
///     }
/// # fn rgb_to_hsl(r: u8, g: u8, b: u8) -> [u8; 3] { [0, 0, 0] }
/// # fn hsl_to_rgb(h: u8, s: u8, l: u8) -> [u8; 3] { [0, 0, 0] }
/// }
/// ```
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

    /// Run this configured `Check`, starting from the default `T` value, using
    /// `T`'s default mutator to create new `T`s, and checking that the given
    /// `property` function returns `Ok(_)` for all of them.
    pub fn run_with_defaults<T, S>(
        &self,
        property: impl FnMut(&T) -> std::result::Result<(), S>,
    ) -> CheckResult<T>
    where
        T: Clone + Debug + Default + DefaultMutator,
        S: ToString,
    {
        self.run(m::default::<T>(), T::default(), property)
    }

    /// Run this configured `Check`, starting with the given `initial_value`,
    /// using the given `mutator` to create new `T`s, and checking that the
    /// given `property` function returns `Ok(_)` for all of them.
    pub fn run<M, T, S>(
        &self,
        mut mutator: M,
        initial_value: T,
        mut property: impl FnMut(&T) -> std::result::Result<(), S>,
    ) -> CheckResult<T>
    where
        M: Mutator<T>,
        T: Clone + Debug,
        S: ToString,
    {
        let mut context = MutationContext::default();
        let mut value = initial_value;

        for _ in 0..self.iters {
            match panic::catch_unwind(panic::AssertUnwindSafe(|| property(&value))) {
                Ok(Ok(())) => {}
                Ok(Err(message)) => {
                    return self.shrink(mutator, value, property, message.to_string());
                }
                Err(_) => {
                    return self.shrink(mutator, value, property, "<panicked>".into());
                }
            }

            match mutator.mutate(&mut context, &mut value) {
                Ok(()) => {}
                Err(e) if e.is_mutator_exhausted() => return Ok(()),
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
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
        eprintln!("failed on input {value:?}: {message}");
        if self.shrink_iters == 0 {
            return Err(CheckFailure { value, message }.into());
        }

        eprintln!("shrinking for {} iters...", self.shrink_iters);

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
                    eprintln!("got mutator error during shrinking, ignoring: {e}");
                    continue;
                }

                Ok(()) => {}
            }

            match panic::catch_unwind(panic::AssertUnwindSafe(|| property(&candidate))) {
                Ok(Ok(())) => {
                    // Not a failure, throw away this candidate and try another
                    // mutation.
                }
                Ok(Err(msg)) => {
                    message = msg.to_string();
                    eprintln!("got failure for shrunken input {value:?}: {message}");
                    value = candidate;
                }
                Err(_) => {
                    message = "<panicked>".to_string();
                    eprintln!("got failure for shrunken input {value:?}: {message}");
                    value = candidate;
                }
            }
        }

        eprintln!("shrunk failing input down to {value:?}");
        Err(CheckFailure { value, message }.into())
    }
}
