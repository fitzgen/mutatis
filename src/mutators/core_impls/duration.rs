use crate::error::ResultExt;

use super::*;

/// A mutator for `core::time::Duration` values.
///
/// See the [`duration()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct Duration<S, N> {
    secs_mutator: S,
    nanos_mutator: N,
}

/// Create a new mutator for `core::time::Duration` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use core::time::Duration;
///
/// let mut mutator = m::duration(m::u64(), m::u32());
/// let mut session = Session::new().seed(1337);
///
/// let mut value = Duration::from_secs(60);
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = 1ns
/// //     value = 13087425514377831989.999999999s
/// //     value = 60s
/// //     value = 18446744073709551615.999999999s
/// //     value = 0ns
/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
pub fn duration<S, N>(secs_mutator: S, nanos_mutator: N) -> Duration<S, N> {
    Duration {
        secs_mutator,
        nanos_mutator,
    }
}

impl<S, N> Mutate<core::time::Duration> for Duration<S, N>
where
    S: Mutate<u64>,
    N: Mutate<u32>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::time::Duration) -> Result<()> {
        c.mutation(|ctx| {
            let mut secs = value.as_secs();
            let mut nanos = value.subsec_nanos();

            let result_a = ctx
                .mutate_with(&mut self.secs_mutator, &mut secs)
                .ignore_exhausted();
            let result_b = ctx
                .mutate_with(&mut self.nanos_mutator, &mut nanos)
                .ignore_exhausted();

            // Clamp nanos to valid range.
            let nanos = nanos.min(999_999_999);

            *value = core::time::Duration::new(secs, nanos);
            result_a.or(result_b)
        })?;

        // Special values.
        if c.shrink() {
            c.mutation(|_| Ok(*value = core::time::Duration::ZERO))?;
        } else {
            c.mutation(|_| Ok(*value = core::time::Duration::ZERO))?;
            c.mutation(|_| Ok(*value = core::time::Duration::MAX))?;
            // Still unstable:
            //
            // c.mutation(|_| Ok(*value = core::time::Duration::from_weeks(1)))?;
            // c.mutation(|_| Ok(*value = core::time::Duration::from_days(1)))?;
            c.mutation(|_| Ok(*value = core::time::Duration::from_hours(1)))?;
            c.mutation(|_| Ok(*value = core::time::Duration::from_mins(1)))?;
            c.mutation(|_| Ok(*value = core::time::Duration::from_secs(1)))?;
            c.mutation(|_| Ok(*value = core::time::Duration::from_millis(1)))?;
            c.mutation(|_| Ok(*value = core::time::Duration::from_micros(1)))?;
            c.mutation(|_| Ok(*value = core::time::Duration::from_nanos(1)))?;
        }

        Ok(())
    }
}

impl<S, N> Generate<core::time::Duration> for Duration<S, N>
where
    S: Generate<u64>,
    N: Generate<u32>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::time::Duration> {
        let secs = self.secs_mutator.generate(ctx)?;
        let mut nanos = self.nanos_mutator.generate(ctx)?;
        if nanos > 999_999_999 {
            nanos = 999_999_999;
        }
        Ok(core::time::Duration::new(secs, nanos))
    }
}

impl DefaultMutate for core::time::Duration {
    type DefaultMutate = Duration<U64, U32>;
}
