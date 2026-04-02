use super::*;

/// A mutator for `core::sync::atomic::AtomicBool` values.
///
/// See the [`atomic_bool()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct AtomicBool {
    inner: Bool,
}

/// Create a new mutator for `core::sync::atomic::AtomicBool` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use core::sync::atomic;
///
/// let mut mutator = m::atomic_bool();
/// let mut session = Session::new();
///
/// let mut value = atomic::AtomicBool::new(true);
/// for _ in 0..3 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {:?}", value.get_mut());
/// }
///
/// // Example output:
/// //
/// //     value = false
/// //     value = true
/// //     value = false
/// # Ok(())
/// # }
/// ```
pub fn atomic_bool() -> AtomicBool {
    AtomicBool {
        inner: Bool { _private: () },
    }
}

impl Mutate<core::sync::atomic::AtomicBool> for AtomicBool {
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut core::sync::atomic::AtomicBool,
    ) -> Result<()> {
        self.inner.mutate(c, value.get_mut())
    }
}

impl Generate<core::sync::atomic::AtomicBool> for AtomicBool {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::sync::atomic::AtomicBool> {
        Ok(core::sync::atomic::AtomicBool::new(
            self.inner.generate(ctx)?,
        ))
    }
}

impl DefaultMutate for core::sync::atomic::AtomicBool {
    type DefaultMutate = AtomicBool;
}

/// A mutator for `core::sync::atomic::AtomicIsize` values.
///
/// See the [`atomic_isize()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct AtomicIsize {
    inner: Isize,
}

/// Create a new mutator for `core::sync::atomic::AtomicIsize` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use core::sync::atomic;
///
/// let mut mutator = m::atomic_isize();
/// let mut session = Session::new();
///
/// let mut value = atomic::AtomicIsize::new(42);
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {:?}", value.get_mut());
/// }
///
/// // Example output:
/// //
/// //     value = -856288970159000865
/// //     value = -650290152212679770
/// //     value = -3756681622550471740
/// //     value = -3564556516857939785
/// //     value = -4296005045519081769
/// # Ok(())
/// # }
/// ```
pub fn atomic_isize() -> AtomicIsize {
    AtomicIsize {
        inner: Isize { _private: () },
    }
}

impl Mutate<core::sync::atomic::AtomicIsize> for AtomicIsize {
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut core::sync::atomic::AtomicIsize,
    ) -> Result<()> {
        self.inner.mutate(c, value.get_mut())
    }
}

impl Generate<core::sync::atomic::AtomicIsize> for AtomicIsize {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::sync::atomic::AtomicIsize> {
        Ok(core::sync::atomic::AtomicIsize::new(
            self.inner.generate(ctx)?,
        ))
    }
}

impl DefaultMutate for core::sync::atomic::AtomicIsize {
    type DefaultMutate = AtomicIsize;
}

/// A mutator for `core::sync::atomic::AtomicUsize` values.
///
/// See the [`atomic_usize()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct AtomicUsize {
    inner: Usize,
}

/// Create a new mutator for `core::sync::atomic::AtomicUsize` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use core::sync::atomic;
///
/// let mut mutator = m::atomic_usize();
/// let mut session = Session::new();
///
/// let mut value = atomic::AtomicUsize::new(42);
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {:?}", value.get_mut());
/// }
///
/// // Example output:
/// //
/// //     value = 14409728900593595992
/// //     value = 10855441405403376060
/// //     value = 16404010271637318453
/// //     value = 2142568799442609111
/// //     value = 17556112587539089552
/// # Ok(())
/// # }
/// ```
pub fn atomic_usize() -> AtomicUsize {
    AtomicUsize {
        inner: Usize { _private: () },
    }
}

impl Mutate<core::sync::atomic::AtomicUsize> for AtomicUsize {
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut core::sync::atomic::AtomicUsize,
    ) -> Result<()> {
        self.inner.mutate(c, value.get_mut())
    }
}

impl Generate<core::sync::atomic::AtomicUsize> for AtomicUsize {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<core::sync::atomic::AtomicUsize> {
        Ok(core::sync::atomic::AtomicUsize::new(
            self.inner.generate(ctx)?,
        ))
    }
}

impl DefaultMutate for core::sync::atomic::AtomicUsize {
    type DefaultMutate = AtomicUsize;
}
