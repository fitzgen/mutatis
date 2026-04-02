use super::*;

/// The default mutator for `CString` values.
///
/// See the [`c_string()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct CStringMutator {
    _private: (),
}

/// Create a new mutator for `CString` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::ffi::CString;
///
/// let mut value = CString::new("hello").unwrap();
///
/// let mut mutator = m::c_string();
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = "he\xe5lo"
/// //     value = "he\x15lo"
/// //     value = "he\x15l"
/// //     value = "hel"
/// //     value = ""
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn c_string() -> CStringMutator {
    CStringMutator { _private: () }
}

/// Generate a random non-zero byte.
fn gen_nonzero_byte(ctx: &mut Context) -> u8 {
    let b = ctx.rng().gen_u8();
    if b == 0 {
        1
    } else {
        b
    }
}

impl Mutate<alloc::ffi::CString> for CStringMutator {
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut alloc::ffi::CString) -> Result<()> {
        let bytes = value.clone().into_bytes();

        // Mutate a random byte (ensuring it stays non-zero).
        if !bytes.is_empty() {
            c.mutation(|ctx| {
                let mut bytes = value.clone().into_bytes();
                let index = ctx.rng().gen_index(bytes.len()).unwrap();
                bytes[index] = gen_nonzero_byte(ctx);
                *value = alloc::ffi::CString::new(bytes).unwrap();
                Ok(())
            })?;
        }

        // Insert a random non-zero byte.
        if !c.shrink() {
            c.mutation(|ctx| {
                let mut bytes = value.clone().into_bytes();
                let index = ctx.rng().gen_index(bytes.len() + 1).unwrap();
                bytes.insert(index, gen_nonzero_byte(ctx));
                *value = alloc::ffi::CString::new(bytes).unwrap();
                Ok(())
            })?;
        }

        // Remove a random byte.
        if !bytes.is_empty() {
            c.mutation(|ctx| {
                let mut bytes = value.clone().into_bytes();
                let index = ctx.rng().gen_index(bytes.len()).unwrap();
                bytes.remove(index);
                *value = alloc::ffi::CString::new(bytes).unwrap();
                Ok(())
            })?;
        }

        Ok(())
    }
}

impl Generate<alloc::ffi::CString> for CStringMutator {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<alloc::ffi::CString> {
        let len = ctx.rng().inner().gen_range(0..=20);
        let bytes: alloc::vec::Vec<u8> = (0..len).map(|_| gen_nonzero_byte(ctx)).collect();
        Ok(alloc::ffi::CString::new(bytes).unwrap())
    }
}

impl DefaultMutate for alloc::ffi::CString {
    type DefaultMutate = CStringMutator;
}
