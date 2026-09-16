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
//! reachable from mutating those inputs (bounded by the configured iteration
//! and duration limits). Additionally, you can assert these same properties and
//! invariants are upheld 24/7 with a coverage-guided fuzzer like `libfuzzer` to
//! explore the state space even more thoroughly.
//!
//! # Example
//!
//! ```
//! use mutatis::{check::Check, mutators as m};
//! use std::time::Duration;
//!
//! // Put this in your crate's `#[cfg(test)] mod tests` and annotate it with
//! // `#[test]`.
//! fn test_rgb_to_hsl_to_rgb_round_trip() {
//!     let result = Check::new()
//!         // Check the property on at least 1000 mutated values, and keep
//!         // going for at least 100 milliseconds...
//!         .min_iters(1000)
//!         .min_duration(Duration::from_millis(100))
//!         // ...but never spend longer than a second on it.
//!         .max_duration(Duration::from_secs(1))
//!         // If we find a failing test case, try to shrink it down to a
//!         // minimal failing test case with up to 1000 shrink iterations.
//!         .max_shrink_iters(1000)
//!         // Run the property check!
//!         .run_with(
//!             // The mutator we'll use to generate new values.
//!             m::array(m::mrange(0x00u8..=0xffu8)),
//!             // The initial corpus of values to check and to derive new
//!             // inputs from via mutation.
//!             [
//!                 [0x00, 0x00, 0x00],
//!                 [0xff, 0xff, 0xff],
//!                 [0x66, 0x33, 0x99],
//!             ],
//!             // The property to check: RGB -> HSL -> RGB should be the
//!             // identity function.
//!             |[r, g, b]| {
//!                 let [h, s, l] = rgb_to_hsl(*r, *g, *b);
//!                 let [r2, g2, b2] = hsl_to_rgb(h, s, l);
//!                 if [*r, *g, *b] == [r2, g2, b2] {
//!                     Ok(())
//!                 } else {
//!                     Err("round-trip conversion failed!")
//!                 }
//!             },
//!         );
//!     assert!(result.is_ok());
//! }
//! # // Stand-ins for the real conversions, just so that this example runs.
//! # fn rgb_to_hsl(r: u8, g: u8, b: u8) -> [u8; 3] { [r, g, b] }
//! # fn hsl_to_rgb(h: u8, s: u8, l: u8) -> [u8; 3] { [h, s, l] }
//! # test_rgb_to_hsl_to_rgb_round_trip();
//! ```
//!
//! # Iteration and Duration Limits
//!
//! You can apply both minimum and maximum bounds to the amount of work a
//! [`Check`] does. It can be the case that these configured bounds conflict:
//! for example, the minimum number of iterations has not been satisfied but the
//! maximum wall-time duration has been exceeded. In these cases, **the minimums
//! always take priority**, so you don't silently do less checking on a slow
//! machine (e.g. in CI or when running under MIRI).
//!
//! For more details, see:
//!
//! * [`Check::min_iters`]
//! * [`Check::max_iters`]
//! * [`Check::iters`]
//! * [`Check::min_duration`]
//! * [`Check::max_duration`]
//! * [`Check::duration`]
//! * [`Check::max_shrink_iters`]
//! * [`Check::max_shrink_duration`]

use super::*;
use crate::log;
use crate::mutators as m;
use std::fmt::{self, Debug};
use std::panic;
use std::prelude::v1::*;
use std::time::{Duration, Instant};

const DEFAULT_MIN_ITERS: usize = 1000;
const DEFAULT_MAX_DURATION: Duration = Duration::from_secs(1);

const DEFAULT_MAX_FACTOR: u32 = 10;
const DEFAULT_SHRINK_FACTOR: u32 = 4;

const SEED_ENV_VAR: &str = "MUTATIS_CHECK_SEED";

fn seed_from_env() -> Option<u64> {
    match std::env::var(SEED_ENV_VAR) {
        Ok(s) if s.trim().is_empty() => None,
        Ok(s) => Some(parse_seed(&s)),
        Err(std::env::VarError::NotPresent) => None,
        Err(e) => panic!("{SEED_ENV_VAR}: {e}"),
    }
}

fn parse_seed(s: &str) -> u64 {
    s.trim()
        .parse()
        .unwrap_or_else(|e| panic!("{SEED_ENV_VAR}={s:?} is not a valid u64: {e}"))
}

/// The result of running a check.
///
/// If the check passes, this is `Ok(())`.
///
/// If the check fails, this is `Err(`[`CheckError::Failed`]`(_))` with the
/// failing test case and an error message.
///
/// If the initial corpus is empty, this is
/// `Err(`[`CheckError::EmptyCorpus`]`)`.
///
/// If there is some other kind of error while running the check, for example if
/// a [`Mutate`] implementation does not support the given [`Session`]
/// configuration, then this is
/// `Err(`[`CheckError::MutatorError`]`(_))`.
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
///     .run_with(
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
    min_iters: Option<usize>,
    max_iters: Option<usize>,
    min_duration: Option<Duration>,
    max_duration: Option<Duration>,
    max_shrink_iters: Option<usize>,
    max_shrink_duration: Option<Duration>,
    seed: Option<u64>,
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
            min_iters: None,
            max_iters: None,
            min_duration: None,
            max_duration: None,
            max_shrink_iters: None,
            max_shrink_duration: None,
            seed: None,
        }
    }

    /// Configure the minimum number of test iterations to run.
    ///
    /// A check always runs at least this many iterations, even when that means
    /// exceeding the configured [maximum duration][Check::max_duration] or
    /// [maximum number of iterations][Check::max_iters].
    ///
    /// The one exception is an exhausted mutator: a check cannot keep going
    /// when there is nothing left to mutate. See
    /// [`run_with`][Check::run_with].
    pub fn min_iters(&mut self, min_iters: usize) -> &mut Check {
        self.min_iters = Some(min_iters);
        self
    }

    /// Configure the maximum number of test iterations to run.
    ///
    /// Once the minimum number of iterations and the minimum duration have both
    /// been reached, a check stops as soon as it has run this many iterations.
    pub fn max_iters(&mut self, max_iters: usize) -> &mut Check {
        self.max_iters = Some(max_iters);
        self
    }

    /// Configure the exact number of test iterations to run.
    ///
    /// This is shorthand for setting both the [minimum][Check::min_iters] and
    /// [maximum][Check::max_iters] number of iterations to the same value.
    pub fn iters(&mut self, iters: usize) -> &mut Check {
        self.min_iters(iters).max_iters(iters)
    }

    /// Configure the minimum duration to run the check for.
    ///
    /// A check always runs for at least this long, even when that means
    /// exceeding the configured [maximum number of
    /// iterations][Check::max_iters] or [maximum
    /// duration][Check::max_duration].
    pub fn min_duration(&mut self, min_duration: Duration) -> &mut Check {
        self.min_duration = Some(min_duration);
        self
    }

    /// Configure the maximum duration to run the check for.
    ///
    /// Once the minimum number of iterations and the minimum duration have both
    /// been reached, a check stops as soon as it has run for this long.
    ///
    /// Note that this does not interrupt an in-progress property evaluation;
    /// the limit is only checked between iterations.
    pub fn max_duration(&mut self, max_duration: Duration) -> &mut Check {
        self.max_duration = Some(max_duration);
        self
    }

    /// Configure the exact duration to run the check for.
    ///
    /// This is shorthand for setting both the [minimum][Check::min_duration]
    /// and [maximum][Check::max_duration] duration to the same value.
    pub fn duration(&mut self, duration: Duration) -> &mut Check {
        self.min_duration(duration).max_duration(duration)
    }

    /// Configure the maximum number of attempts to shrink a failing input
    /// before reporting the failure.
    ///
    /// Shrinking has no minimums: it stops as soon as it hits this limit, the
    /// [maximum shrink duration][Check::max_shrink_duration], or an exhausted
    /// mutator.
    ///
    /// Setting this to zero disables shrinking.
    pub fn max_shrink_iters(&mut self, max_shrink_iters: usize) -> &mut Check {
        self.max_shrink_iters = Some(max_shrink_iters);
        self
    }

    /// Configure the maximum duration to spend shrinking a failing input before
    /// reporting the failure.
    ///
    /// Shrinking has no minimums: it stops as soon as it hits this limit, the
    /// [maximum number of shrink iterations][Check::max_shrink_iters], or an
    /// exhausted mutator.
    ///
    /// Setting this to zero disables shrinking.
    pub fn max_shrink_duration(&mut self, max_shrink_duration: Duration) -> &mut Check {
        self.max_shrink_duration = Some(max_shrink_duration);
        self
    }

    #[deprecated(since = "0.5.4", note = "renamed to `Check::max_shrink_iters`")]
    #[doc(hidden)]
    pub fn shrink_iters(&mut self, shrink_iters: usize) -> &mut Check {
        self.max_shrink_iters(shrink_iters)
    }

    fn min_iters_or_default(&self) -> usize {
        self.min_iters.unwrap_or(DEFAULT_MIN_ITERS)
    }

    fn max_iters_or_default(&self) -> usize {
        self.max_iters.unwrap_or_else(|| {
            self.min_iters_or_default()
                .saturating_mul(DEFAULT_MAX_FACTOR as usize)
        })
    }

    fn min_duration_or_default(&self) -> Duration {
        self.min_duration.unwrap_or(Duration::ZERO)
    }

    fn max_duration_or_default(&self) -> Duration {
        self.max_duration.unwrap_or_else(|| {
            self.min_duration.map_or(DEFAULT_MAX_DURATION, |min| {
                min.saturating_mul(DEFAULT_MAX_FACTOR)
            })
        })
    }

    fn max_shrink_iters_or_default(&self) -> usize {
        self.max_shrink_iters.unwrap_or_else(|| {
            self.min_iters_or_default()
                .saturating_mul(DEFAULT_SHRINK_FACTOR as usize)
        })
    }

    fn max_shrink_duration_or_default(&self) -> Duration {
        self.max_shrink_duration.unwrap_or_else(|| {
            self.max_duration_or_default()
                .saturating_mul(DEFAULT_SHRINK_FACTOR)
        })
    }

    fn seed_or_default(&self) -> u64 {
        self.seed
            .or_else(seed_from_env)
            .unwrap_or_else(crate::rng::next_default_seed)
    }

    /// Configure the RNG seed used for mutation.
    ///
    /// When this is not set, the `MUTATIS_CHECK_SEED` environment variable is
    /// consulted; unless empty, it must parse as a `u64` or the check panics.
    /// Failing that, a seed is chosen automatically.
    ///
    /// Every run logs the seed it used at debug level, so a failing check can
    /// be reproduced by setting it here or in the environment.
    pub fn seed(&mut self, seed: u64) -> &mut Check {
        self.seed = Some(seed);
        self
    }

    /// Run this configured `Check` with a default initial `T` value and the
    /// default mutator.
    ///
    /// This is a convenience method that is equivalent to calling
    /// [`run_with`][Check::run_with] with
    /// [`m::default::<T>()`][crate::mutators::default] and
    /// [`[T::default()]`][core::default::Default::default] as the only value in
    /// the initial corpus.
    pub fn run<T, S>(
        &self,
        property: impl FnMut(&T) -> std::result::Result<(), S>,
    ) -> CheckResult<T>
    where
        T: Clone + Debug + Default + DefaultMutate,
        S: ToString,
    {
        self.run_with(m::default::<T>(), [T::default()], property)
    }

    /// Run this configured `Check` with the given corpus and mutator.
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
    /// corpus and for each mutated value. If the property returns an error or
    /// panics, the check is considered to have failed and the failing input
    /// value is shrunk down to a minimal failing value. You can configure how
    /// much effort is put into shrinking via the
    /// [`max_shrink_iters`][Check::max_shrink_iters] and
    /// [`max_shrink_duration`][Check::max_shrink_duration] methods.
    ///
    /// How many mutated values are checked is determined by the configured
    /// iteration and duration limits; see [the module-level
    /// documentation][crate::check#iteration-and-duration-limits]. The duration
    /// limits cover this whole method, including checking the initial corpus,
    /// but only mutated values count towards the iteration limits: one
    /// iteration is one property evaluation on one mutated value.
    ///
    /// If, in the process of running checks, the mutator exhausts all potential
    /// mutations it can apply to the corpus, then `run_with` stops and reports
    /// success, even if the minimum iteration count or duration have not been
    /// met.
    pub fn run_with<M, T, S>(
        &self,
        mut mutator: M,
        initial_corpus: impl IntoIterator<Item = T>,
        mut property: impl FnMut(&T) -> std::result::Result<(), S>,
    ) -> CheckResult<T>
    where
        M: Mutate<T>,
        T: Clone + Debug,
        S: ToString,
    {
        let start = Instant::now();

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

        // Second, run the check on mutated values derived from the corpus
        // until our configured iteration and duration limits say we are done.
        let seed = self.seed_or_default();
        log::debug!("using RNG seed {seed}");
        let mut session = Session::new().seed(seed);

        let min_iters = self.min_iters_or_default();
        let max_iters = self.max_iters_or_default();
        let min_duration = self.min_duration_or_default();
        let max_duration = self.max_duration_or_default();

        let mut iters = 0_usize;
        loop {
            // We keep going while any minimum is unmet, and otherwise until we
            // reach a maximum.
            if iters >= min_iters {
                let elapsed = start.elapsed();
                let mins_met = elapsed >= min_duration;
                let max_reached = iters >= max_iters || elapsed >= max_duration;
                if mins_met && max_reached {
                    break;
                }
            }

            let index = session.context.rng().gen_index(corpus.len()).unwrap();

            match session.mutate_with(&mut mutator, &mut corpus[index]) {
                Ok(()) => {}
                Err(e) if e.is_exhausted() => {
                    corpus.swap_remove(index);
                    if corpus.is_empty() {
                        return Ok(());
                    }
                    continue;
                }
                Err(e) => return Err(e.into()),
            }

            if let Err(msg) = Self::check_one(&corpus[index], &mut property) {
                return self.shrink(mutator, corpus[index].clone(), property, msg);
            }

            iters += 1;
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
        M: Mutate<T>,
        T: Clone + Debug,
        S: ToString,
    {
        let start = Instant::now();

        log::warn!("failed on input {value:?}: {message}");

        let max_shrink_iters = self.max_shrink_iters_or_default();
        let max_shrink_duration = self.max_shrink_duration_or_default();
        if max_shrink_iters == 0 || max_shrink_duration.is_zero() {
            return Err(CheckFailure { value, message }.into());
        }

        log::debug!("shrinking for up to {max_shrink_iters} iters or {max_shrink_duration:?}...");

        let seed = self.seed_or_default();
        log::debug!("shrinking with RNG seed {seed}");
        let mut session = Session::new().seed(seed).shrink(true);

        for _ in 0..max_shrink_iters {
            if start.elapsed() >= max_shrink_duration {
                log::debug!("reached maximum shrink duration; stopping shrinking");
                break;
            }

            let mut candidate = value.clone();

            match session.mutate_with(&mut mutator, &mut candidate) {
                // If the mutator is exhausted, then don't keep trying to shrink
                // the input and just report the final error.
                Err(e) if e.is_exhausted() => break,

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

    /// Run `check` over a trivially-passing property, sleeping for `delay` on
    /// each call, and return the number of mutation iterations it performed.
    fn count_iters(check: &mut Check, delay: Duration) -> usize {
        let mut calls = 0_usize;
        check
            .run_with(m::u32(), [0u32], |_: &u32| -> Result<(), String> {
                calls += 1;
                if !delay.is_zero() {
                    std::thread::sleep(delay);
                }
                Ok(())
            })
            .unwrap();
        // The first call is the initial corpus's single value, which does not
        // count as a mutation iteration.
        calls - 1
    }

    /// A property that fails for every value greater than or equal to ten.
    fn less_than_ten(x: &u8) -> Result<(), &'static str> {
        if *x < 10 {
            Ok(())
        } else {
            Err("expected < 10")
        }
    }

    #[test]
    fn check_shrinking_negative_ints() {
        let failure = check()
            .max_shrink_iters(1000)
            .run_with(m::i32(), [-1i32, i32::MIN], |x: &i32| {
                if *x >= 0 {
                    Ok(())
                } else {
                    Err("negative")
                }
            })
            .unwrap_err()
            .unwrap_failed();

        assert!(
            failure.value < 0,
            "{} is not a counterexample",
            failure.value
        );
    }

    #[test]
    fn check_shrinking_out_of_range() {
        let failure = check()
            .max_shrink_iters(1000)
            .run_with(m::mrange(10i32..=20i32), [5i32], |_: &i32| {
                Err::<(), &str>("always fails")
            })
            .unwrap_err()
            .unwrap_failed();

        assert!(
            (10..=20).contains(&failure.value),
            "{} escaped the mutator's range",
            failure.value
        );
    }

    #[test]
    fn parse_seed_accepts_decimal_and_trims() {
        assert_eq!(parse_seed("42"), 42);
        assert_eq!(parse_seed(" 42 "), 42);
        assert_eq!(parse_seed("42\n"), 42);
        assert_eq!(parse_seed("0"), 0);
        assert_eq!(parse_seed(&u64::MAX.to_string()), u64::MAX);
    }

    #[test]
    #[should_panic(expected = "MUTATIS_CHECK_SEED")]
    fn parse_seed_rejects_non_numeric() {
        parse_seed("abc");
    }

    #[test]
    #[should_panic(expected = "MUTATIS_CHECK_SEED")]
    fn parse_seed_rejects_overflow() {
        // `u64::MAX + 1`.
        parse_seed("18446744073709551616");
    }

    #[test]
    fn seed_env_var_is_consulted() {
        assert_eq!(std::env::var_os(SEED_ENV_VAR), None);

        // Unset and unconfigured: the default sequence, which advances.
        assert_ne!(
            Check::new().seed_or_default(),
            Check::new().seed_or_default()
        );

        std::env::set_var(SEED_ENV_VAR, "1234");
        assert_eq!(Check::new().seed_or_default(), 1234);
        // An explicit seed still wins.
        assert_eq!(Check::new().seed(7).seed_or_default(), 7);

        // Set but empty is ignored, so the default sequence resumes.
        for empty in ["", " ", "\n"] {
            std::env::set_var(SEED_ENV_VAR, empty);
            assert_ne!(
                Check::new().seed_or_default(),
                Check::new().seed_or_default()
            );
        }
        std::env::remove_var(SEED_ENV_VAR);

        assert_ne!(Check::new().seed_or_default(), 1234);
    }

    /// A mutator that is exhausted for zero and otherwise increments the value.
    struct ExhaustedForZero;

    impl Mutate<u32> for ExhaustedForZero {
        fn mutate(&mut self, c: &mut Candidates<'_>, value: &mut u32) -> crate::Result<()> {
            if *value == 0 {
                return Ok(());
            }
            c.mutation(|_| Ok(*value += 1))
        }
    }

    #[test]
    fn check_iters_is_exact() {
        assert_eq!(count_iters(check().iters(37), Duration::ZERO), 37);
    }

    #[test]
    fn check_min_iters_beats_max_iters() {
        let iters = count_iters(check().min_iters(25).max_iters(5), Duration::ZERO);
        assert_eq!(iters, 25);
    }

    #[test]
    fn check_min_iters_beats_max_duration() {
        let iters = count_iters(
            check().min_iters(10).max_duration(Duration::from_millis(1)),
            Duration::from_millis(1),
        );
        assert_eq!(iters, 10);
    }

    #[test]
    fn check_min_duration_runs_past_min_iters() {
        let min_duration = Duration::from_millis(20);
        let start = Instant::now();
        let iters = count_iters(
            check().min_iters(1).min_duration(min_duration),
            Duration::from_millis(1),
        );
        let elapsed = start.elapsed();
        assert!(elapsed >= min_duration, "only ran for {elapsed:?}");
        assert!(
            iters > 1,
            "ran {iters} iters, expected more than min_iters=1"
        );
    }

    #[test]
    fn check_duration_runs_for_at_least_that_long() {
        let duration = Duration::from_millis(20);
        let start = Instant::now();
        count_iters(check().min_iters(0).duration(duration), Duration::ZERO);
        let elapsed = start.elapsed();
        assert!(elapsed >= duration, "only ran for {elapsed:?}");
    }

    #[test]
    fn check_max_shrink_duration_bounds_shrinking() {
        let max_shrink_duration = Duration::from_millis(20);
        let start = Instant::now();

        // Without the duration limit, an unlimited shrink iteration count
        // would never terminate.
        let failure = check()
            .max_shrink_iters(usize::MAX)
            .max_shrink_duration(max_shrink_duration)
            .run_with(m::u8(), [u8::MAX], less_than_ten)
            .unwrap_err()
            .unwrap_failed();

        let elapsed = start.elapsed();
        assert!(
            elapsed >= max_shrink_duration,
            "only shrank for {elapsed:?}"
        );

        // How far shrinking gets in a fixed amount of time depends on how fast
        // the machine is, so only assert that it made some progress.
        assert!(
            (10..u8::MAX).contains(&failure.value),
            "shrank to {}, expected something in 10..255",
            failure.value
        );
    }

    #[test]
    fn check_zero_max_shrink_iters_disables_shrinking() {
        let failure = check()
            .max_shrink_iters(0)
            .run_with(m::u8(), [u8::MAX], less_than_ten)
            .unwrap_err()
            .unwrap_failed();
        assert_eq!(failure.value, u8::MAX);
    }

    #[test]
    fn check_zero_max_shrink_duration_disables_shrinking() {
        let failure = check()
            .max_shrink_duration(Duration::ZERO)
            .run_with(m::u8(), [u8::MAX], less_than_ten)
            .unwrap_err()
            .unwrap_failed();
        assert_eq!(failure.value, u8::MAX);
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_shrink_iters_sets_max_shrink_iters() {
        let failure = check()
            .shrink_iters(0)
            .run_with(m::u8(), [u8::MAX], less_than_ten)
            .unwrap_err()
            .unwrap_failed();
        assert_eq!(failure.value, u8::MAX);
    }

    #[test]
    fn check_run_with_okay() {
        check()
            .run_with(m::just(true), [true], |b: &bool| {
                if *b {
                    Ok(())
                } else {
                    Err("expected true!")
                }
            })
            .unwrap();
    }

    #[test]
    fn check_run_with_fail() {
        let failure = check()
            .run_with(m::bool(), [true], |b: &bool| {
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
    fn check_run_with_empty_corpus() {
        let result = check().run_with(m::bool(), [], |b: &bool| {
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
    fn check_run_with_exhausted_value_is_dropped_and_not_rechecked() {
        const ITERS: usize = 100;

        // The exhausted value is last in the corpus, so dropping it from the
        // corpus leaves the index we chose out of bounds.
        let mut seen = Vec::new();
        check()
            .iters(ITERS)
            .run_with(ExhaustedForZero, [1, 0], |x: &u32| -> Result<(), String> {
                seen.push(*x);
                Ok(())
            })
            .unwrap();

        // Two values in the initial corpus, plus one property check per
        // iteration: an exhausted mutation does not consume an iteration.
        assert_eq!(seen.len(), 2 + ITERS);

        // Every mutation increments, and a dropped value is never checked
        // again, so we should never see the same value twice.
        let mut sorted = seen.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), seen.len(), "checked a value twice: {seen:?}");
    }

    #[test]
    fn check_run_with_exhausted_corpus_succeeds() {
        // `m::unit()` is always exhausted, so the corpus empties out and the
        // check stops early, before reaching any of its limits.
        let mut calls = 0_usize;
        check()
            .run_with(m::unit(), [(), ()], |_: &()| -> Result<(), String> {
                calls += 1;
                Ok(())
            })
            .unwrap();

        // Once per initial corpus value, and zero mutation iterations.
        assert_eq!(calls, 2);
    }

    #[test]
    fn check_run_with_fail_and_shrink() {
        let failure = check()
            .max_shrink_iters(1000)
            .run_with(m::u8(), [u8::MAX], |x: &u8| {
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
    fn check_run_with_fail_on_panic() {
        let result = check().run_with(m::bool(), [true], |_: &bool| -> Result<(), String> {
            std::panic!("oh no!")
        });
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CheckError::Failed(_)));
    }

    #[test]
    fn check_run_with_fail_on_panic_and_shrink() {
        let failure = check()
            .max_shrink_iters(1000)
            .run_with(m::u8(), [u8::MAX], |x: &u8| -> Result<(), String> {
                assert!(*x < 10);
                Ok(())
            })
            .unwrap_err()
            .unwrap_failed();

        assert_eq!(failure.value, 10);
        assert_eq!(failure.message, "<panicked>");
    }
}
