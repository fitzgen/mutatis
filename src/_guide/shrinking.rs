/*!

# Shrinking Test Cases

You can configure a [`Session`][crate::Session] to only perform mutations that
"shrink" their given values. Compared to the original input, a shrunken value is
simpler and less complex, has fewer members inside its inner `Vec`s and other
container types, serializes to fewer bytes, and etc...

When paired with an oracle, property, or predicate function, shrinking makes
test-case reduction easy! You can automate finding the smallest and
easiest-to-understand input that still triggers a bug in your code.

## Example

```rust
# fn foo() -> mutatis::Result<()> {
use mutatis::{mutators as m, Mutate, Session};

// Configure mutation to only shrink the input.
let mut session = Session::new().shrink(true);

let mut value = u32::MAX;
for _ in 0..10 {
    session.mutate(&mut value)?;
    println!("shrunken value is {value}");
}

// Example output:
//
//     shrunken value is 1682887620
//     shrunken value is 363687628
//     shrunken value is 259482126
//     shrunken value is 49968845
//     shrunken value is 12933345
//     shrunken value is 9334495
//     shrunken value is 124077
//     shrunken value is 12325
//     shrunken value is 9732
//     shrunken value is 3837
# Ok(())
# }
# foo().unwrap()
```

## Shrinking and Manual `Mutate` Implementations

When implementing [`Mutate`][crate::Mutate] by hand, rather than relying on the
[`mutatis::mutators`][crate::mutators] module's combinators or the
[`#[derive(Mutate)]`][crate::_guide::derive_macro] macro, be sure to check
whether you're inside a shrinking session and adjust your candidate mutations
appropriately. In general, checking for shrinking is often something that only
collections and "leaf" `Mutate` implementations need to worry about; if you are
simply delegating to sub-mutators then they can likely take responsibility for
shrinking.

Here is an example mutator that produces powers of two. When we are shrinking,
it only produces powers of two that are smaller than the original value:

```rust
# fn foo() -> mutatis::Result<()> {
use mutatis::{
    mutators as m, Error, Generate, Mutate, Session, Candidates,
    Result,
};

/// A mutator that mutates `u32`s into powers of two.
pub struct PowersOfTwo;

impl Mutate<u32> for PowersOfTwo {
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut u32,
    ) -> Result<()> {
        // If we should only shrink the value, then only generate powers of two
        // less than the input. Otherwise, generate any power of two `u32`.
        let max_log2 = if c.shrink() {
            match value.ilog2().checked_sub(value.is_power_of_two() as u32) {
                Some(x) => x,
                // There are no more powers of two to shrink to, so early return
                // and do not register any candidate mutations.
                None => return Ok(()),
            }
        } else {
            31
        };

        c.mutation(|context| {
            // Choose a random `log2(value)` between 0 and `max_log2`, inclusive.
            let log2 = m::range(0..=max_log2).generate(context)?;

            // value = 2^log2(value) = 1 << log2(value)
            *value = 1 << log2;

            Ok(())
        })?;

        Ok(())
    }
}

let mut value = u32::MAX;

// Configure mutation to only shrink the input.
let mut session = Session::new().shrink(true);

for _ in 0..4 {
    session.mutate_with(&mut PowersOfTwo, &mut value)?;
    println!("shrunken value is {value}");
}

// Example output:
//
//     shrunken value is 8388608
//     shrunken value is 8192
//     shrunken value is 128
//     shrunken value is 16
# Ok(())
# }
# mutatis::error::ResultExt::ignore_exhausted(foo()).unwrap()
```

## See Also

* [`Session::shrink`][crate::Session::shrink]: Configure whether a mutation
  session should only shrink or not.

* [`Candidates::shrink`][crate::Candidates::shrink]: While enumerating candidate
  mutations inside a `Mutate::mutate` implementation, determine whether you
  should only register candidate mutations that shrink the original value.

* [`Context::shrink`][crate::Context::shrink]: While applying a mutation, check
  whether the current mutation should only shrink the original value.

* [`Check::shrink_iters`][crate::check::Check::shrink_iters]: Configure the
  number of attempts to shrink a failing input before reporting the failure.

 */
