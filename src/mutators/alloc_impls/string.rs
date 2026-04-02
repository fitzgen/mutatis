use super::*;

/// The default mutator for `String` values.
///
/// See the [`string()`] function to create new instances and for example usage.
#[derive(Clone, Debug, Default)]
pub struct StringMutator<C> {
    char_mutator: C,
}

/// Create a new mutator for `String` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut value = String::from("hello");
///
/// let mut mutator = m::string(m::ascii_char());
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = "he}lo"
/// //     value = "he}loK:2h\u{1e}|R\u{6}u\u{1c}"
/// //     value = "he}l"
/// //     value = "he}l\\"
/// //     value = "h e}l\\"
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn string<C>(char_mutator: C) -> StringMutator<C> {
    StringMutator { char_mutator }
}

impl<C> Mutate<alloc::string::String> for StringMutator<C>
where
    C: Generate<char>,
{
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut alloc::string::String) -> Result<()> {
        // Remove a random character.
        if !value.is_empty() {
            c.mutation(|ctx| {
                let index = ctx.rng().gen_index(value.chars().count()).unwrap();
                let byte_index = value.char_indices().nth(index).unwrap().0;
                value.remove(byte_index);
                Ok(())
            })?;
        }

        // Truncate.
        if !value.is_empty() {
            c.mutation(|ctx| {
                let index = ctx.rng().gen_index(value.chars().count()).unwrap();
                let byte_index = value.char_indices().nth(index).unwrap().0;
                value.truncate(byte_index);
                Ok(())
            })?;
        }

        // Replace a random character.
        if !value.is_empty() {
            c.mutation(|ctx| {
                let count = value.chars().count();
                let index = ctx.rng().gen_index(count).unwrap();
                let (byte_index, old_char) = value.char_indices().nth(index).unwrap();
                let new_char = self.char_mutator.generate(ctx)?;
                let end = byte_index + old_char.len_utf8();
                let mut buf = [0u8; 4];
                let s = new_char.encode_utf8(&mut buf);
                value.replace_range(byte_index..end, s);
                Ok(())
            })?;
        }

        // Insert a random character at a random position.
        if !c.shrink() {
            c.mutation(|ctx| {
                let count = value.chars().count();
                let index = ctx.rng().gen_index(count + 1).unwrap();
                let byte_index = if index == count {
                    value.len()
                } else {
                    value.char_indices().nth(index).unwrap().0
                };
                let ch = self.char_mutator.generate(ctx)?;
                value.insert(byte_index, ch);
                Ok(())
            })?;
        }

        // Append a character.
        if !c.shrink() {
            c.mutation(|ctx| {
                let ch = self.char_mutator.generate(ctx)?;
                value.push(ch);
                Ok(())
            })?;
        }

        // Push multiple random characters.
        if !c.shrink() {
            c.mutation(|ctx| {
                let count = ctx.rng().inner().gen_range(2..=10);
                for _ in 0..count {
                    let ch = self.char_mutator.generate(ctx)?;
                    value.push(ch);
                }
                Ok(())
            })?;
        }

        Ok(())
    }
}

impl<C> Generate<alloc::string::String> for StringMutator<C>
where
    C: Generate<char>,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<alloc::string::String> {
        let len = ctx.rng().inner().gen_range(0..=20);
        let mut s = alloc::string::String::with_capacity(len);
        for _ in 0..len {
            s.push(self.char_mutator.generate(ctx)?);
        }
        Ok(s)
    }
}

impl DefaultMutate for alloc::string::String {
    type DefaultMutate = StringMutator<Char>;
}
