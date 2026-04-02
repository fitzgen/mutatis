use super::*;

/// A mutator for `core::ops::Range<T>` values.
///
/// See the [`range()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct Range<M> {
    mutator: M,
}

/// Create a new mutator for `core::ops::Range<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::range(m::u32());
/// let mut session = Session::new();
///
/// let mut value = 10u32..20u32;
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = 0..1279744118
/// //     value = -1044336733..1279744118
/// //     value = -1044336733..-1116147001
/// //     value = 1028247545..-1116147001
/// //     value = 1028247545..825715763
/// # Ok(())
/// # }
/// ```
pub fn range<M>(mutator: M) -> Range<M> {
    Range { mutator }
}

impl<M, T> Mutate<core::ops::Range<T>> for Range<M>
where
    M: Mutate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::ops::Range<T>) -> Result<()> {
        self.mutator.mutate(c, &mut value.start)?;
        self.mutator.mutate(c, &mut value.end)?;
        Ok(())
    }
}

impl<M, T> Generate<core::ops::Range<T>> for Range<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::ops::Range<T>> {
        let start = self.mutator.generate(ctx)?;
        let end = self.mutator.generate(ctx)?;
        Ok(start..end)
    }
}

impl<T> DefaultMutate for core::ops::Range<T>
where
    T: DefaultMutate,
{
    type DefaultMutate = Range<T::DefaultMutate>;
}

// --- RangeFrom<T> ---

/// A mutator for `core::ops::RangeFrom<T>` values.
///
/// See the [`range_from()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct RangeFrom<M> {
    mutator: M,
}

/// Create a new mutator for `core::ops::RangeFrom<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::range_from(m::u32());
/// let mut session = Session::new();
///
/// let mut value = 10u32..;
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = -1147817089..
/// //     value = -1265999069..
/// //     value = -1235153252..
/// //     value = 1465721499..
/// //     value = -1647191276..
/// # Ok(())
/// # }
/// ```
pub fn range_from<M>(mutator: M) -> RangeFrom<M> {
    RangeFrom { mutator }
}

impl<M, T> Mutate<core::ops::RangeFrom<T>> for RangeFrom<M>
where
    M: Mutate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::ops::RangeFrom<T>) -> Result<()> {
        self.mutator.mutate(c, &mut value.start)
    }
}

impl<M, T> Generate<core::ops::RangeFrom<T>> for RangeFrom<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::ops::RangeFrom<T>> {
        let start = self.mutator.generate(ctx)?;
        Ok(start..)
    }
}

impl<T> DefaultMutate for core::ops::RangeFrom<T>
where
    T: DefaultMutate,
{
    type DefaultMutate = RangeFrom<T::DefaultMutate>;
}

// --- RangeInclusive<T> ---

/// A mutator for `core::ops::RangeInclusive<T>` values.
///
/// See the [`range_inclusive()`] function to create new instances and for
/// example usage.
#[derive(Clone, Debug, Default)]
pub struct RangeInclusive<M> {
    mutator: M,
}

/// Create a new mutator for `core::ops::RangeInclusive<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::range_inclusive(m::u32());
/// let mut session = Session::new();
///
/// let mut value = 10u32..=20u32;
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = 0..=1514844130
/// //     value = -602108615..=1514844130
/// //     value = 230176042..=1514844130
/// //     value = 230176042..=-134193539
/// //     value = -1759104937..=-134193539
/// # Ok(())
/// # }
/// ```
pub fn range_inclusive<M>(mutator: M) -> RangeInclusive<M> {
    RangeInclusive { mutator }
}

impl<M, T> Mutate<core::ops::RangeInclusive<T>> for RangeInclusive<M>
where
    M: Mutate<T>,
    T: Default + Clone,
{
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut core::ops::RangeInclusive<T>,
    ) -> Result<()> {
        // RangeInclusive fields are private, so we mutate by generating new
        // start/end values inside mutation closures.
        c.mutation(|ctx| {
            let mut start = value.start().clone();
            let result = ctx.mutate_with(&mut self.mutator, &mut start);
            let end = value.end().clone();
            *value = start..=end;
            result
        })?;
        c.mutation(|ctx| {
            let start = value.start().clone();
            let mut end = value.end().clone();
            let result = ctx.mutate_with(&mut self.mutator, &mut end);
            *value = start..=end;
            result
        })?;
        Ok(())
    }
}

impl<M, T> Generate<core::ops::RangeInclusive<T>> for RangeInclusive<M>
where
    M: Generate<T>,
    T: Default + Clone,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::ops::RangeInclusive<T>> {
        let start = self.mutator.generate(ctx)?;
        let end = self.mutator.generate(ctx)?;
        Ok(start..=end)
    }
}

impl<T> DefaultMutate for core::ops::RangeInclusive<T>
where
    T: DefaultMutate + Default + Clone,
{
    type DefaultMutate = RangeInclusive<T::DefaultMutate>;
}

/// A mutator for `core::ops::RangeTo<T>` values.
///
/// See the [`range_to()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct RangeTo<M> {
    mutator: M,
}

/// Create a new mutator for `core::ops::RangeTo<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::range_to(m::u32());
/// let mut session = Session::new();
///
/// let mut value = ..20u32;
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = ..-132877274
/// //     value = ..1348424364
/// //     value = ..1905959479
/// //     value = ..1111619826
/// //     value = ..1766421991
/// # Ok(())
/// # }
/// ```
pub fn range_to<M>(mutator: M) -> RangeTo<M> {
    RangeTo { mutator }
}

impl<M, T> Mutate<core::ops::RangeTo<T>> for RangeTo<M>
where
    M: Mutate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::ops::RangeTo<T>) -> Result<()> {
        self.mutator.mutate(c, &mut value.end)
    }
}

impl<M, T> Generate<core::ops::RangeTo<T>> for RangeTo<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::ops::RangeTo<T>> {
        let end = self.mutator.generate(ctx)?;
        Ok(..end)
    }
}

impl<T> DefaultMutate for core::ops::RangeTo<T>
where
    T: DefaultMutate,
{
    type DefaultMutate = RangeTo<T::DefaultMutate>;
}

/// A mutator for `core::ops::RangeToInclusive<T>` values.
///
/// See the [`range_to_inclusive()`] function to create new instances and for
/// example usage.
#[derive(Clone, Debug, Default)]
pub struct RangeToInclusive<M> {
    mutator: M,
}

/// Create a new mutator for `core::ops::RangeToInclusive<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::range_to_inclusive(m::u32());
/// let mut session = Session::new();
///
/// let mut value = ..=20u32;
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = ..=1947902770
/// //     value = ..=1390957595
/// //     value = ..=720619876
/// //     value = ..=1511374761
/// //     value = ..=2008834718
/// # Ok(())
/// # }
/// ```
pub fn range_to_inclusive<M>(mutator: M) -> RangeToInclusive<M> {
    RangeToInclusive { mutator }
}

impl<M, T> Mutate<core::ops::RangeToInclusive<T>> for RangeToInclusive<M>
where
    M: Mutate<T>,
{
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut core::ops::RangeToInclusive<T>,
    ) -> Result<()> {
        self.mutator.mutate(c, &mut value.end)
    }
}

impl<M, T> Generate<core::ops::RangeToInclusive<T>> for RangeToInclusive<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::ops::RangeToInclusive<T>> {
        let end = self.mutator.generate(ctx)?;
        Ok(..=end)
    }
}

impl<T> DefaultMutate for core::ops::RangeToInclusive<T>
where
    T: DefaultMutate,
{
    type DefaultMutate = RangeToInclusive<T::DefaultMutate>;
}

// --- Bound<T> ---

/// A mutator for `core::ops::Bound<T>` values.
///
/// See the [`bound()`] function to create new instances and for example usage.
#[derive(Clone, Debug, Default)]
pub struct Bound<M> {
    mutator: M,
}

/// Create a new mutator for `core::ops::Bound<T>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use core::ops::Bound;
///
/// let mut mutator = m::bound(m::u32());
/// let mut session = Session::new();
///
/// let mut value = Bound::Included(42u32);
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = Included(-299147346)
/// //     value = Included(1194369574)
/// //     value = Excluded(300006767)
/// //     value = Included(-1814544879)
/// //     value = Excluded(-1183271801)
/// # Ok(())
/// # }
/// ```
pub fn bound<M>(mutator: M) -> Bound<M> {
    Bound { mutator }
}

impl<M, T> Mutate<core::ops::Bound<T>> for Bound<M>
where
    M: Generate<T>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut core::ops::Bound<T>) -> Result<()> {
        match value {
            core::ops::Bound::Included(v) => {
                self.mutator.mutate(c, v)?;
                if !c.shrink() {
                    c.mutation(|ctx| {
                        *value = core::ops::Bound::Excluded(self.mutator.generate(ctx)?);
                        Ok(())
                    })?;
                }
                c.mutation(|_| {
                    *value = core::ops::Bound::Unbounded;
                    Ok(())
                })?;
            }
            core::ops::Bound::Excluded(v) => {
                self.mutator.mutate(c, v)?;
                c.mutation(|ctx| {
                    *value = core::ops::Bound::Included(self.mutator.generate(ctx)?);
                    Ok(())
                })?;
                c.mutation(|_| {
                    *value = core::ops::Bound::Unbounded;
                    Ok(())
                })?;
            }
            core::ops::Bound::Unbounded => {
                c.mutation(|ctx| {
                    *value = core::ops::Bound::Included(self.mutator.generate(ctx)?);
                    Ok(())
                })?;
                if !c.shrink() {
                    c.mutation(|ctx| {
                        *value = core::ops::Bound::Excluded(self.mutator.generate(ctx)?);
                        Ok(())
                    })?;
                }
            }
        }
        Ok(())
    }
}

impl<M, T> Generate<core::ops::Bound<T>> for Bound<M>
where
    M: Generate<T>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::ops::Bound<T>> {
        match ctx.rng().gen_u8() % 3 {
            0 => Ok(core::ops::Bound::Included(self.mutator.generate(ctx)?)),
            1 => Ok(core::ops::Bound::Excluded(self.mutator.generate(ctx)?)),
            _ => Ok(core::ops::Bound::Unbounded),
        }
    }
}

impl<T> DefaultMutate for core::ops::Bound<T>
where
    T: DefaultMutate,
    T::DefaultMutate: Generate<T>,
{
    type DefaultMutate = Bound<T::DefaultMutate>;
}
