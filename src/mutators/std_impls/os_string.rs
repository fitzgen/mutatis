use super::*;
use std::ffi::OsString;

/// The default mutator for `OsString` values.
///
/// See the [`os_string()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct OsStringMutator {
    string_mutator: StringMutator<Char>,
}

/// Create a new mutator for `OsString` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::ffi::OsString;
///
/// let mut value = OsString::from("hello");
///
/// let mut mutator = m::os_string();
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = ".."
/// //     value = "\\"
/// //     value = "\\"
/// //     value = "\\🭪"
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn os_string() -> OsStringMutator {
    OsStringMutator::default()
}

impl Mutate<OsString> for OsStringMutator {
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut OsString) -> Result<()> {
        // Remove a random character.
        if let Some(s) = value.to_str() {
            let len = s.len();
            if len != 0 {
                c.mutation(|ctx| {
                    let i = ctx.rng().gen_index(len).unwrap();
                    let s: alloc::string::String = value
                        .to_str()
                        .unwrap()
                        .char_indices()
                        .filter_map(|(j, c)| if i == j { None } else { Some(c) })
                        .collect();
                    *value = OsString::from(s);
                    Ok(())
                })?;
            }
        }

        // Add a random character.
        if !c.shrink() {
            c.mutation(|ctx| {
                let ch = ctx.rng().gen_char();
                let mut buf = [0u8; 4];
                let s = ch.encode_utf8(&mut buf);
                value.push(s);
                Ok(())
            })?;
        }

        // Special: empty.
        if !value.is_empty() {
            c.mutation(|_ctx| {
                *value = OsString::new();
                Ok(())
            })?;
        }

        // Special: ".".
        c.mutation(|_ctx| {
            *value = OsString::from(".");
            Ok(())
        })?;

        // Special: "..".
        c.mutation(|_ctx| {
            *value = OsString::from("..");
            Ok(())
        })?;

        // Special: "/".
        c.mutation(|_ctx| {
            *value = OsString::from("/");
            Ok(())
        })?;

        // Special: "\\".
        c.mutation(|_ctx| {
            *value = OsString::from("\\");
            Ok(())
        })?;

        Ok(())
    }
}

impl Generate<OsString> for OsStringMutator {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<OsString> {
        let s = self.string_mutator.generate(ctx)?;
        Ok(OsString::from(s))
    }
}

impl DefaultMutate for OsString {
    type DefaultMutate = OsStringMutator;
}
